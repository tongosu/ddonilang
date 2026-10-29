use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ddonirang_core::realms::{MultiRealmManager, RealmStepInput, ThreadMode};
use hex::encode as hex_encode;
use serde::{Deserialize, Serialize};

use crate::artifact_output::write_text_artifact_atomic;

#[derive(Debug, Deserialize)]
struct RealmsTestInput {
    master_seed: u64,
    realm_count: usize,
    steps: u64,
    step_batch: Vec<RealmsStepInput>,
}

#[derive(Debug, Deserialize)]
struct RealmsStepInput {
    realm_id: usize,
    delta: i64,
}

#[derive(Debug, Serialize)]
struct RealmsTestOutput {
    realm_count: usize,
    steps: u64,
    state_hashes: Vec<String>,
}

fn evaluate_realms_test(text: &str, threads: usize) -> Result<String, String> {
    let input: RealmsTestInput =
        serde_json::from_str(text).map_err(|err| format!("E_REALM_INPUT {}", err))?;
    if input.realm_count == 0 {
        return Err("E_REALM_INPUT realm_count must be > 0".to_string());
    }
    let thread_mode = if threads <= 1 {
        ThreadMode::Seq
    } else {
        ThreadMode::Rayon(threads)
    };
    let mut manager = MultiRealmManager::new(input.realm_count, input.master_seed, thread_mode);
    let batch: Vec<RealmStepInput> = input
        .step_batch
        .iter()
        .map(|item| RealmStepInput {
            realm_id: item.realm_id,
            delta: item.delta,
        })
        .collect();
    for _ in 0..input.steps {
        manager.step_batch(&batch)?;
    }
    let state_hashes = manager
        .state_hashes()
        .iter()
        .map(|hash| format!("blake3:{}", hex_encode(hash.as_bytes())))
        .collect::<Vec<_>>();
    let output = RealmsTestOutput {
        realm_count: input.realm_count,
        steps: input.steps,
        state_hashes,
    };
    serde_json::to_string_pretty(&output).map_err(|err| format!("E_REALM_OUTPUT {}", err))
}

pub fn run_realms_test(path: &Path, threads: usize, out: Option<&Path>) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    let json = evaluate_realms_test(&text, threads)?;
    if let Some(out_path) = out {
        write_text_artifact_atomic(out_path, &format!("{json}\n"))?;
    }
    println!("{json}");
    Ok(())
}

#[derive(Debug, Clone)]
pub struct GoldenRunnerOptions {
    pub packs: Vec<String>,
    pub all: bool,
    pub record: bool,
    pub update: bool,
}

#[derive(Debug, Clone)]
pub struct SmokeRunnerOptions {
    pub packs: Vec<String>,
    pub update: bool,
    pub skip_ui_common: bool,
    pub skip_wrapper: bool,
}

fn find_workspace_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|err| format!("E_TEST_CWD {}", err))?;
    loop {
        let has_pack_runner = dir.join("tests").join("run_pack_golden.py").exists();
        let has_smoke_runner = dir
            .join("tests")
            .join("run_seamgrim_wasm_smoke.py")
            .exists();
        if has_pack_runner && has_smoke_runner {
            return Ok(dir);
        }
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent.to_path_buf();
    }
    Err("E_TEST_WORKSPACE_ROOT tests runner 파일을 찾지 못했습니다.".to_string())
}

fn run_python_runner(root: &Path, script_rel: &str, args: &[String]) -> Result<(), String> {
    let script_path = root.join(script_rel);
    if !script_path.exists() {
        return Err(format!("E_TEST_RUNNER_MISSING {}", script_path.display()));
    }
    let mut command = Command::new("python");
    command.arg(&script_path);
    command.args(args);
    command.current_dir(root);
    command.stdin(Stdio::null());
    let status = command
        .status()
        .map_err(|err| format!("E_TEST_RUNNER_SPAWN {}: {}", script_path.display(), err))?;
    if !status.success() {
        return Err(format!(
            "E_TEST_RUNNER_FAILED {} exit_code={}",
            script_path.display(),
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

pub fn run_pack_golden_runner(options: GoldenRunnerOptions) -> Result<(), String> {
    let root = find_workspace_root()?;
    let mut args = Vec::new();
    if options.all {
        args.push("--all".to_string());
    }
    if options.record {
        args.push("--record".to_string());
    }
    if options.update {
        args.push("--update".to_string());
    }
    args.extend(options.packs.iter().cloned());
    run_python_runner(&root, "tests/run_pack_golden.py", &args)
}

pub fn run_wasm_smoke_runner(options: SmokeRunnerOptions) -> Result<(), String> {
    let root = find_workspace_root()?;
    let mut args = Vec::new();
    if options.update {
        args.push("--update".to_string());
    }
    if options.skip_ui_common {
        args.push("--skip-ui-common".to_string());
    }
    if options.skip_wrapper {
        args.push("--skip-wrapper".to_string());
    }
    args.extend(options.packs.iter().cloned());
    run_python_runner(&root, "tests/run_seamgrim_wasm_smoke.py", &args)
}

#[cfg(test)]
mod tests {
    use super::evaluate_realms_test;

    const INPUT: &str = r#"{
  "master_seed": 7,
  "realm_count": 2,
  "steps": 3,
  "step_batch": [
    { "realm_id": 0, "delta": 2 },
    { "realm_id": 1, "delta": -1 }
  ]
}"#;

    #[test]
    fn realm_test_is_deterministic_across_thread_modes() {
        let sequential = evaluate_realms_test(INPUT, 1).expect("sequential realm test");
        let parallel = evaluate_realms_test(INPUT, 2).expect("parallel realm test");
        assert_eq!(sequential, parallel);
        assert!(sequential.contains("\"realm_count\": 2"));
        assert!(sequential.contains("\"steps\": 3"));
        assert_eq!(sequential.matches("blake3:").count(), 2);
    }

    #[test]
    fn realm_test_rejects_zero_realms() {
        let error = evaluate_realms_test(
            r#"{"master_seed":0,"realm_count":0,"steps":0,"step_batch":[]}"#,
            1,
        )
        .expect_err("zero realms must fail closed");
        assert_eq!(error, "E_REALM_INPUT realm_count must be > 0");
    }

    #[test]
    fn realm_test_rejects_malformed_input() {
        let error = evaluate_realms_test("{}", 1).expect_err("missing fields must fail closed");
        assert!(error.starts_with("E_REALM_INPUT "), "{error}");
    }
}
