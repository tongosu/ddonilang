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
const PROGRAMS: [(&str, &str); 4] = [
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/01_요금_계산기.ddn"),
        "2800\n2100\n",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/02_조건과_반복.ddn"),
        "작은 제곱\n작은 제곱\n큰 제곱\n큰 제곱\n30\n",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/03_자동_함수표.ddn"),
        "0\n0\n2\n1\n1\n4\n2\n4\n10\n3\n9\n20\n",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/04_시간단계_시뮬레이션.ddn"),
        "1\n2\n3\n2\n5\n4\n3\n9\n5\n4\n14\n6\n",
    ),
];
const INVALID_PROGRAMS: [(&str, &str); 5] = [
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/01_요금_타입오류.ddn"),
        "E_RUNTIME_TYPE_MISMATCH",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/02_반복_원소타입오류.ddn"),
        // RD-352: the declared homogeneous List has a statically known bad
        // Text member. Reject before execution, preserving the taught error
        // and the no-output/atomic failure assertions below.
        "E_CHECK_TYPE_MISMATCH",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/03_함수표_인자누락.ddn"),
        "E_CALL_REQUIRED_PIN_MISSING",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/04_시뮬레이션_인자중복.ddn"),
        "핀 '위치'에 인자가 중복되었습니다",
    ),
    (
        include_str!("../../solutions/seamgrim_ui_mvp/alpha1/잘못된입력/05_조사_모호오류.ddn"),
        "조사 '을'가 모호합니다",
    ),
];
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ddn-alpha1-product-{}-{}",
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
fn all_official_clis_run_the_four_editable_alpha1_programs() {
    for binary in BINS {
        for (index, (source, expected)) in PROGRAMS.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("program-{index}.ddn"), source);
            let output = run(binary, &["실행", text(&source_path), "--source-profile", PROFILE]);
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(String::from_utf8(output.stdout).unwrap(), *expected);
        }
    }
}

#[test]
fn alpha1_programs_keep_canonical_roundtrip_stable() {
    for binary in BINS {
        for (index, (source, _)) in PROGRAMS.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("source-{index}.ddn"), source);
            let first = run(binary, &["개발", "정본", text(&source_path), "--source-profile", PROFILE]);
            assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
            let canonical_path = scratch.source(
                &format!("canonical-{index}.ddn"),
                &String::from_utf8_lossy(&first.stdout),
            );
            let second = run(binary, &["canon", text(&canonical_path), "--source-profile", PROFILE]);
            assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
            assert_eq!(first.stdout, second.stdout);
        }
    }
}

#[test]
fn all_official_clis_reject_representative_alpha1_input_errors_atomically() {
    for binary in BINS {
        for (index, (source, expected_code)) in INVALID_PROGRAMS.iter().enumerate() {
            let scratch = Scratch::new();
            let source_path = scratch.source(&format!("invalid-{index}.ddn"), source);
            let output = run(binary, &["실행", text(&source_path), "--source-profile", PROFILE]);
            assert!(!output.status.success(), "{expected_code}가 성공으로 처리됨");
            assert!(output.stdout.is_empty(), "{expected_code} 실패 뒤 출력이 게시됨");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(expected_code),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn all_official_clis_admit_short_literal_definition_and_reject_legacy_or_typed_mismatch() {
    const SHORT_LITERAL: &str = "값 := 100.\n값 보여주기.\n";
    const LEGACY_EQUALS: &str = "값 = 100.\n값 보여주기.\n";
    const TYPED_MISMATCH: &str = "매마디:움직씨 := {\n    값:수 := \"글\".\n    값 보여주기.\n}\n";

    for binary in BINS {
        let scratch = Scratch::new();
        let source_path = scratch.source("short-literal.ddn", SHORT_LITERAL);
        let output = run(binary, &["실행", text(&source_path), "--source-profile", PROFILE]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "100\n");

        let canonical = run(binary, &["개발", "정본", text(&source_path), "--source-profile", PROFILE]);
        assert!(canonical.status.success(), "{}", String::from_utf8_lossy(&canonical.stderr));
        assert!(String::from_utf8_lossy(&canonical.stdout).contains("값 := 100."));

        let legacy_path = scratch.source("legacy-equals.ddn", LEGACY_EQUALS);
        let legacy = run(binary, &["실행", text(&legacy_path), "--source-profile", PROFILE]);
        assert!(!legacy.status.success());
        assert!(legacy.stdout.is_empty());
        assert!(String::from_utf8_lossy(&legacy.stderr).contains("E_V25_DEFINITION_MIGRATION_REQUIRED"));

        let mismatch_path = scratch.source("typed-mismatch.ddn", TYPED_MISMATCH);
        let mismatch = run(binary, &["실행", text(&mismatch_path), "--source-profile", PROFILE]);
        assert!(!mismatch.status.success());
        assert!(mismatch.stdout.is_empty());
        assert!(String::from_utf8_lossy(&mismatch.stderr).contains("E_RUNTIME_TYPE_MISMATCH"));
    }
}
