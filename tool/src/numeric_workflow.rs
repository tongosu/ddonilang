use std::fs;
use std::path::Path;

use ddonirang_numeric::{complete_factor, factor_job_from_json, step_factor_job, to_detjson};

use crate::artifact_output::{
    require_file_artifact_target, write_text_artifact_atomic, write_text_artifacts_atomic,
};

pub fn run_factor_complete(input: &str, out: Option<&Path>) -> Result<(), String> {
    let outcome = complete_factor(input)?;
    let result_text = to_detjson(&outcome.result)?;
    if let Some(out_dir) = out {
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("E_NUMERIC_FACTOR_OUT_DIR {} {}", out_dir.display(), e))?;
        let result_path = out_dir.join("factor_result.detjson");
        let job_path = out_dir.join("factor_job.detjson");
        require_file_artifact_target(&result_path)
            .map_err(|e| format!("E_NUMERIC_FACTOR_RESULT_WRITE {e}"))?;
        require_file_artifact_target(&job_path)
            .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_WRITE {e}"))?;
        let job_text = to_detjson(&outcome.job)?;
        write_text_artifacts_atomic(&[
            (result_path.as_path(), result_text.as_str()),
            (job_path.as_path(), job_text.as_str()),
        ])
        .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_WRITE {e}"))?;
    }
    println!("{result_text}");
    Ok(())
}

pub fn run_factor_step(
    input: Option<&str>,
    resume: Option<&Path>,
    budget_ops: u64,
    job_out: Option<&Path>,
) -> Result<(), String> {
    let job = match resume {
        Some(path) => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_READ {} {}", path.display(), e))?;
            factor_job_from_json(&text)?
        }
        None => {
            let value = input.ok_or_else(|| {
                "E_NUMERIC_FACTOR_INPUT_REQUIRED input 또는 --resume 필요".to_string()
            })?;
            ddonirang_numeric::new_factor_job(value)?
        }
    };
    let outcome = step_factor_job(job, budget_ops.max(1))?;
    if let Some(path) = job_out {
        require_file_artifact_target(path)
            .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_WRITE {e}"))?;
        write_text_artifact_atomic(path, &to_detjson(&outcome.job)?)
            .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_WRITE {e}"))?;
    }
    println!("{}", to_detjson(&outcome.result)?);
    Ok(())
}
