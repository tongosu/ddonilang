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
        include_str!("fixtures/alpha_callable_function_table_loop_zero.ddn"),
        "",
    ),
    (
        include_str!("fixtures/alpha_callable_function_table_loop_one.ddn"),
        "4\n16\n34\n큼\n",
    ),
    (
        include_str!("fixtures/alpha_callable_function_table_loop_many.ddn"),
        "0\n0\n2\n작음\n1\n1\n4\n작음\n2\n4\n10\n큼\n3\n9\n20\n큼\n",
    ),
];
const FAILURE: &str =
    include_str!("fixtures/alpha_callable_function_table_loop_failure.ddn");
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ddn-alpha-function-table-loop-{}-{}",
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
fn official_cli_builds_function_tables_from_zero_one_and_many_inputs() {
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
fn official_cli_keeps_loop_function_table_canon_stable() {
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
fn official_cli_discards_all_output_when_a_middle_iteration_call_fails() {
    for binary in BINS {
        let scratch = Scratch::new();
        let source_path = scratch.source("failure.ddn", FAILURE);
        let output = run(
            binary,
            &["실행", text(&source_path), "--source-profile", PROFILE],
        );
        assert!(!output.status.success(), "중간 호출 실패가 성공으로 처리됨");
        assert!(output.stdout.is_empty(), "중간 실패 전에 만든 출력이 게시됨");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("E_RESULTPLACE_UNASSIGNED"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
