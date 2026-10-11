use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const PROFILE: &str = "v1-core-v25";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tool crate must have a repository parent")
        .to_path_buf()
}

fn invoke(binary: &str, operation: &str, source: &Path) -> Output {
    let mut command = Command::new(binary);
    command
        .current_dir(repository_root())
        .arg(operation)
        .arg(source)
        .arg("--source-profile")
        .arg(PROFILE);
    if matches!(operation, "실행" | "run") {
        command.arg("--json");
    }
    command.output()
        .expect("official DDN CLI must start")
}

fn json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "official CLI stdout must be JSON: {error}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn d_pack_exec_r1_uses_the_official_check_and_run_path() {
    let root = repository_root();
    let pack = root.join("pack/ddn_exec_r1");
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(pack.join("manifest.detjson")).expect("read D-PACK manifest"),
    )
    .expect("parse D-PACK manifest");
    assert_eq!(manifest["schema"], "ddn.exec.pack.v1");
    assert_eq!(manifest["pack_id"], "D-PACK-EXEC-R1");
    assert_eq!(manifest["source_profile_identity"], PROFILE);

    let cases = manifest["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 11);
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let source = pack.join(case["source"].as_str().expect("case source"));
        let check = invoke(env!("CARGO_BIN_EXE_ddonilang"), "검사", &source);
        let check_json = json_stdout(&check);

        if let Some(expected) = case["expected_diagnostic"].as_str() {
            assert!(
                !check.status.success(),
                "{id} must fail closed during check"
            );
            assert_eq!(check_json["outcome"], "REJECTED", "{id}");
            assert_eq!(check_json["diagnostics"][0]["code"], expected, "{id}");
            let run = invoke(env!("CARGO_BIN_EXE_ddonilang"), "실행", &source);
            assert!(
                !run.status.success(),
                "{id} must fail before runtime publication"
            );
            assert!(
                String::from_utf8_lossy(&run.stderr).contains(expected),
                "{id} run diagnostic must preserve {expected}: {}",
                String::from_utf8_lossy(&run.stderr)
            );
            continue;
        }

        assert!(
            check.status.success(),
            "{id} check failed: {}",
            String::from_utf8_lossy(&check.stdout)
        );
        assert_eq!(check_json["outcome"], "ACCEPTED", "{id}");

        let run = invoke(env!("CARGO_BIN_EXE_ddonilang"), "실행", &source);
        assert!(
            run.status.success(),
            "{id} run failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = json_stdout(&run);
        assert_eq!(
            run_json["schema"],
            "ddn.ddonirang_tool.shared_run_summary.v1"
        );
        assert_eq!(run_json["source_profile_identity"], PROFILE);
        assert_eq!(
            run_json["output_log_texts"][0], case["expected_output"],
            "{id}"
        );

        if case["deterministic_replay"] == true {
            let repeated = invoke(env!("CARGO_BIN_EXE_ddonilang"), "실행", &source);
            assert!(repeated.status.success(), "{id} repeat must succeed");
            assert_eq!(
                run.stdout, repeated.stdout,
                "{id} output must be byte-stable"
            );
        }
    }
}

#[test]
fn three_official_names_execute_gp01_with_identical_bytes() {
    let source = repository_root().join("pack/ddn_exec_r1/cases/EXEC-010.ddn");
    let binaries = [
        env!("CARGO_BIN_EXE_ddonilang"),
        env!("CARGO_BIN_EXE_또니랑"),
        env!("CARGO_BIN_EXE_ddonirang"),
    ];
    let outputs = binaries
        .iter()
        .map(|binary| invoke(binary, "실행", &source))
        .collect::<Vec<_>>();
    for output in &outputs {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(json_stdout(output)["output_log_texts"][0], "7");
    }
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
    assert_eq!(outputs[0].stdout, outputs[2].stdout);
}
