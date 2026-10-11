use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const PROFILE: &str = "v1-core-v25";
const BINS: [&str; 3] = [
    env!("CARGO_BIN_EXE_ddonilang"),
    env!("CARGO_BIN_EXE_또니랑"),
    env!("CARGO_BIN_EXE_ddonirang"),
];
const SUCCESSES: [(&str, &str); 3] = [
    (
        include_str!("fixtures/alpha_callable_value_chain.ddn"),
        "7\n",
    ),
    (include_str!("fixtures/alpha_callable_nested.ddn"), "7\n"),
    (
        include_str!("fixtures/alpha_callable_function_table.ddn"),
        "0\n0\n1\n1\n2\n4\n3\n9\n",
    ),
];
const FAILURES: [(&str, &str); 3] = [
    (
        include_str!("fixtures/alpha_callable_unassigned.ddn"),
        "E_RESULTPLACE_UNASSIGNED",
    ),
    (
        include_str!("fixtures/alpha_callable_read_forbidden.ddn"),
        "E_RESULTPLACE_READ_FORBIDDEN",
    ),
    (
        include_str!("fixtures/alpha_callable_wrong_result_type.ddn"),
        "E_RUNTIME_TYPE_MISMATCH",
    ),
];
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ddn-alpha-callable-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn source(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, text).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary).args(args).output().unwrap()
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn official_cli_runs_value_chain_nested_call_and_function_table() {
    for binary in BINS {
        for (index, (source, expected)) in SUCCESSES.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("success-{index}.ddn"), source);
            let output = run(
                binary,
                &["실행", text(&source_path), "--source-profile", PROFILE],
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8(output.stdout).unwrap(), *expected);
        }
    }
}

#[test]
fn official_cli_preserves_callable_canon_roundtrip() {
    for binary in BINS {
        for (index, (source, _)) in SUCCESSES.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("source-{index}.ddn"), source);
            let first = run(
                binary,
                &[
                    "개발",
                    "정본",
                    text(&source_path),
                    "--source-profile",
                    PROFILE,
                ],
            );
            assert!(
                first.status.success(),
                "{}",
                String::from_utf8_lossy(&first.stderr)
            );
            let canonical_path = scratch.source(
                &format!("canonical-{index}.ddn"),
                &String::from_utf8_lossy(&first.stdout),
            );
            let second = run(
                binary,
                &["canon", text(&canonical_path), "--source-profile", PROFILE],
            );
            assert!(
                second.status.success(),
                "{}",
                String::from_utf8_lossy(&second.stderr)
            );
            assert_eq!(first.stdout, second.stdout);
        }
    }
}

#[test]
fn official_cli_fails_closed_at_result_place_boundaries() {
    for binary in BINS {
        for (index, (source, code)) in FAILURES.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("failure-{index}.ddn"), source);
            let output = run(
                binary,
                &["실행", text(&source_path), "--source-profile", PROFILE],
            );
            assert!(!output.status.success(), "{code}가 성공으로 잘못 처리됨");
            assert!(output.stdout.is_empty(), "{code} 실패 뒤 성공 출력 게시");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(code),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
