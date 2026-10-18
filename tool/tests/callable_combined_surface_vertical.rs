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
const SUCCESS: &str = include_str!("fixtures/callable_combined_surface_success.ddn");
const FAILURES: [(&str, &str); 3] = [
    (
        include_str!("fixtures/callable_combined_surface_unregistered.ddn"),
        "E_CALL_TAIL_NO_SEED",
    ),
    (
        include_str!("fixtures/callable_combined_surface_duplicate.ddn"),
        "E_CALLABLE_SURFACE_DUPLICATE",
    ),
    (
        include_str!("fixtures/callable_combined_surface_ambiguous.ddn"),
        "E_CALLABLE_SURFACE_AMBIGUOUS",
    ),
];
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ddn-callable-combined-surface-{}-{}",
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
fn all_official_clis_run_preferred_and_alternate_calls_as_one_definition() {
    for binary in BINS {
        let scratch = Scratch::new();
        let source = scratch.source("success.ddn", SUCCESS);
        let output = run(
            binary,
            &["실행", text(&source), "--source-profile", PROFILE],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "5\n");
    }
}

#[test]
fn all_official_clis_preserve_canon_reparse() {
    for binary in BINS {
        let scratch = Scratch::new();
        let source = scratch.source("source.ddn", SUCCESS);
        let first = run(
            binary,
            &["개발", "정본", text(&source), "--source-profile", PROFILE],
        );
        assert!(
            first.status.success(),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );
        let canonical = String::from_utf8(first.stdout.clone()).unwrap();
        assert!(canonical.contains("돕다{도우}:셈수:셈씨"));
        let canonical_path = scratch.source("canonical.ddn", &canonical);
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

#[test]
fn all_official_clis_fail_closed_for_unregistered_duplicate_and_ambiguous_surfaces() {
    for binary in BINS {
        for (index, (source, expected)) in FAILURES.iter().enumerate() {
            let scratch = Scratch::new();
            let path = scratch.source(&format!("failure-{index}.ddn"), source);
            let output = run(binary, &["실행", text(&path), "--source-profile", PROFILE]);
            assert!(!output.status.success(), "{expected}가 성공으로 처리됨");
            assert!(output.stdout.is_empty(), "실패 뒤 출력이 게시됨");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(expected),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn all_official_clis_keep_preferred_family_and_reject_alternate_widening() {
    let definitions = r#"
(값:셈수) 분류{숫자형}:글:셈씨 := { 분류 <- "숫자". }.
(값:글) 분류:글:셈씨 := { 분류 <- "문자". }.
"#;
    let accepted = format!(
        "{definitions}매마디:움직씨 := {{ (값=7) 분류 보여주기. (값=\"가\") 분류 보여주기. (값=7) 숫자형 보여주기. }}."
    );
    let rejected = format!("{definitions}매마디:움직씨 := {{ (값=\"가\") 숫자형 보여주기. }}.");

    for binary in BINS {
        let scratch = Scratch::new();
        let accepted_path = scratch.source("overload-ok.ddn", &accepted);
        let output = run(
            binary,
            &["실행", text(&accepted_path), "--source-profile", PROFILE],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "숫자\n문자\n숫자\n"
        );

        let rejected_path = scratch.source("overload-bad.ddn", &rejected);
        let output = run(
            binary,
            &["실행", text(&rejected_path), "--source-profile", PROFILE],
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("E_ORDINARY_CALLABLE_NO_MATCH"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
