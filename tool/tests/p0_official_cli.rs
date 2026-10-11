//! P0 공식 source 명령의 제품 실행 관문. DDN 원문은 기존 pack 그대로 소비한다.
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
const SOURCE: &str = include_str!(
    "../../pack/v1_core_supported_ingress_w1/fixtures/positive_definition_rebinding.ddn"
);
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ddn-p0-cli-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn source(&self) -> PathBuf {
        let path = self.0.join("원문.ddn");
        fs::write(&path, SOURCE).unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn cli(binary: &str, args: &[&str]) -> Output {
    Command::new(binary).args(args).output().unwrap()
}
fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn failure(output: &Output, code: &str) {
    assert!(!output.status.success(), "성공으로 잘못 처리됨");
    assert!(output.stdout.is_empty(), "실패 뒤 성공 출력 게시");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn p0_lint_patch_cannot_replace_source_or_path_alias() {
    let dir = Scratch::new();
    let source = dir.source();
    let alias = dir.0.join(".").join("원문.ddn");
    for binary in BINS {
        for target in [&source, &alias] {
            let output = cli(
                binary,
                &[
                    "lint",
                    text(&source),
                    "--source-profile",
                    PROFILE,
                    "--suggest-patch",
                    "--out",
                    text(target),
                ],
            );
            failure(&output, "E_CLI_OUTPUT_ALIASES_INPUT");
            assert_eq!(fs::read(&source).unwrap(), SOURCE.as_bytes());
            assert_eq!(
                fs::read_dir(&dir.0).unwrap().count(),
                1,
                "부분 patch/staging 게시"
            );
        }
        let patch = dir.0.join("제안.json");
        let output = cli(
            binary,
            &[
                "lint",
                text(&source),
                "--source-profile",
                PROFILE,
                "--suggest-patch",
                "--out",
                text(&patch),
            ],
        );
        success(&output);
        let _: serde_json::Value = serde_json::from_slice(&fs::read(&patch).unwrap()).unwrap();
        assert_eq!(fs::read(&source).unwrap(), SOURCE.as_bytes());
        fs::remove_file(&patch).unwrap();
    }
}

#[test]
fn p0_source_commands_share_analysis_canon_and_execution() {
    let dir = Scratch::new();
    let source = dir.source();
    let mut expected = None;
    for binary in BINS {
        let mut check_bytes = None;
        for command in ["검사", "check", "짓기", "build"] {
            let output = cli(
                binary,
                &[command, text(&source), "--source-profile", PROFILE],
            );
            success(&output);
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["outcome"], "ACCEPTED");
            if let Some(bytes) = &check_bytes {
                assert_eq!(&output.stdout, bytes);
            }
            check_bytes = Some(output.stdout);
        }
        let canon = cli(
            binary,
            &["개발", "정본", text(&source), "--source-profile", PROFILE],
        );
        success(&canon);
        assert!(canon.stdout.ends_with(b"\n"));
        let canonical_file = dir.0.join("정본.ddn");
        fs::write(&canonical_file, &canon.stdout).unwrap();
        let again = cli(
            binary,
            &["canon", text(&canonical_file), "--source-profile", PROFILE],
        );
        success(&again);
        assert_eq!(canon.stdout, again.stdout);
        let mut run_bytes = None;
        for command in ["실행", "run"] {
            let human = cli(
                binary,
                &[
                    command,
                    text(&source),
                    "--source-profile",
                    PROFILE,
                    "--마디수",
                    "1",
                ],
            );
            success(&human);
            assert_eq!(human.stdout, b"2\n");
            let run = cli(
                binary,
                &[
                    command,
                    text(&source),
                    "--source-profile",
                    PROFILE,
                    "--madi",
                    "1",
                    "--json",
                ],
            );
            success(&run);
            let value: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
            assert_eq!(value["source_profile_identity"], PROFILE);
            if let Some(bytes) = &run_bytes {
                assert_eq!(&run.stdout, bytes);
            }
            run_bytes = Some(run.stdout);
        }
        let observed = (check_bytes.unwrap(), canon.stdout, run_bytes.unwrap());
        if let Some(previous) = &expected {
            assert_eq!(&observed, previous);
        }
        expected = Some(observed);
        assert_eq!(fs::read(&source).unwrap(), SOURCE.as_bytes());
    }
}

#[test]
fn p0_invalid_source_profile_and_runtime_never_publish_artifacts() {
    let dir = Scratch::new();
    let source = dir.source();
    let summary = dir.0.join("실행.json");
    let original = b"previous-complete-output\n";
    fs::write(&summary, original).unwrap();
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pack/v1_core_supported_ingress_w1/fixtures");
    for binary in BINS {
        for command in ["run-maze", "train", "evolve", "unknown"] {
            failure(&cli(binary, &[command]), "E_CLI_COMMAND_UNKNOWN");
        }
        for command in ["check", "build", "canon", "lint", "run"] {
            failure(
                &cli(binary, &[command, text(&source)]),
                "E_SOURCE_PROFILE_REQUIRED",
            );
            let unknown = cli(
                binary,
                &[
                    command,
                    text(&source),
                    "--source-profile",
                    "unknown-profile",
                ],
            );
            if matches!(command, "check" | "build") {
                // 검사/짓기의 stdout은 기존 analysis snapshot 채널이며 실행 출력이 아니다.
                assert!(!unknown.status.success());
                let diagnostic: serde_json::Value =
                    serde_json::from_slice(&unknown.stdout).unwrap();
                assert_eq!(diagnostic["outcome"], "UNSUPPORTED");
                assert_eq!(
                    diagnostic["diagnostics"][0]["code"],
                    "E_SOURCE_PROFILE_UNSUPPORTED"
                );
                for field in [
                    "output_log_texts",
                    "output_rows",
                    "state_hash",
                    "trace_hash",
                    "receipt",
                ] {
                    assert!(diagnostic.get(field).is_none());
                }
            } else {
                failure(&unknown, "E_SOURCE_PROFILE_UNSUPPORTED");
            }
        }
        for (fixture, code) in [
            (
                "legacy_definition_equals.ddn",
                "E_V25_DEFINITION_MIGRATION_REQUIRED",
            ),
            ("direct_recursion.ddn", "E_V1_CORE_RECURSION_UNSUPPORTED"),
            ("mutual_recursion.ddn", "E_V1_CORE_RECURSION_UNSUPPORTED"),
            (
                "strict_selection_out_of_range.ddn",
                "CHARIM_INDEX_OUT_OF_RANGE",
            ),
            ("value_type_mismatch.ddn", "E_RUNTIME_TYPE_MISMATCH"),
        ] {
            failure(
                &cli(
                    binary,
                    &[
                        "run",
                        text(&fixtures.join(fixture)),
                        "--source-profile",
                        PROFILE,
                        "--madi",
                        "1",
                        "--json",
                        "--summary-json",
                        text(&summary),
                    ],
                ),
                code,
            );
            assert_eq!(fs::read(&summary).unwrap(), original);
        }
        let rejected = fixtures.join("legacy_definition_equals.ddn");
        failure(
            &cli(
                binary,
                &[
                    "lint",
                    text(&rejected),
                    "--source-profile",
                    PROFILE,
                    "--suggest-patch",
                    "--out",
                    text(&summary),
                ],
            ),
            "E_V25_DEFINITION_MIGRATION_REQUIRED",
        );
        assert_eq!(fs::read(&summary).unwrap(), original);
        failure(
            &cli(
                binary,
                &[
                    "run",
                    text(&source),
                    "--source-profile",
                    PROFILE,
                    "--summary-json",
                    text(&source),
                ],
            ),
            "E_CLI_OUTPUT_ALIASES_INPUT",
        );
        assert_eq!(fs::read(&source).unwrap(), SOURCE.as_bytes());
        for bytes in [&[0xff, 0xfe][..], b"\0"] {
            let bad = dir.0.join("잘못된.ddn");
            fs::write(&bad, bytes).unwrap();
            let output = cli(
                binary,
                &[
                    "run",
                    text(&bad),
                    "--source-profile",
                    PROFILE,
                    "--summary-json",
                    text(&summary),
                ],
            );
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert_eq!(fs::read(&summary).unwrap(), original);
        }
    }
}

#[test]
fn p0_help_version_collection_and_output_modes_are_name_invariant() {
    let dir = Scratch::new();
    let source = dir.source();
    let summary = dir.0.join("summary.json");
    let collection = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pack/v1_core_supported_ingress_w1/fixtures/language_core_collection.ddn");
    let mut expected = None;
    for binary in BINS {
        let help = cli(binary, &["도움"]);
        success(&help);
        assert_eq!(help.stdout, cli(binary, &["--help"]).stdout);
        let help_text = String::from_utf8(help.stdout.clone()).unwrap();
        for command in ["실행", "검사", "짓기", "정본", "lint"] {
            assert!(help_text.contains(command));
        }
        for internal in ["teul-cli", "run-maze", "train", "evolve"] {
            assert!(!help_text.contains(internal));
        }
        let version = cli(binary, &["판본"]);
        success(&version);
        assert_eq!(version.stdout, cli(binary, &["--version"]).stdout);
        let run = cli(
            binary,
            &[
                "run",
                text(&source),
                "--source-profile",
                PROFILE,
                "--마디수",
                "1",
                "--summary-json",
                text(&summary),
            ],
        );
        success(&run);
        assert_eq!(run.stdout, b"2\n");
        let json = cli(
            binary,
            &[
                "run",
                text(&source),
                "--source-profile",
                PROFILE,
                "--madi",
                "1",
                "--json",
            ],
        );
        success(&json);
        assert_eq!(fs::read(&summary).unwrap(), json.stdout);
        let collection_run = cli(
            binary,
            &[
                "run",
                text(&collection),
                "--source-profile",
                PROFILE,
                "--madi",
                "1",
                "--json",
            ],
        );
        success(&collection_run);
        let observations = (
            help.stdout,
            version.stdout,
            json.stdout,
            collection_run.stdout,
        );
        if let Some(previous) = &expected {
            assert_eq!(&observations, previous);
        }
        expected = Some(observations);
    }
}
