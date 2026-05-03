use std::fs;
use std::path::Path;

use ddonirang_numeric::{complete_factor, factor_job_from_json, step_factor_job, to_detjson};

pub fn run_factor_complete(input: &str, out: Option<&Path>) -> Result<(), String> {
    let outcome = complete_factor(input)?;
    let result_text = to_detjson(&outcome.result)?;
    if let Some(out_dir) = out {
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("E_NUMERIC_FACTOR_OUT_DIR {} {}", out_dir.display(), e))?;
        fs::write(out_dir.join("factor_result.detjson"), &result_text)
            .map_err(|e| format!("E_NUMERIC_FACTOR_RESULT_WRITE {e}"))?;
        fs::write(
            out_dir.join("factor_job.detjson"),
            to_detjson(&outcome.job)?,
        )
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
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("E_NUMERIC_FACTOR_JOB_OUT_DIR {} {}", parent.display(), e)
                })?;
            }
        }
        fs::write(path, to_detjson(&outcome.job)?)
            .map_err(|e| format!("E_NUMERIC_FACTOR_JOB_WRITE {e}"))?;
    }
    println!("{}", to_detjson(&outcome.result)?);
    Ok(())
}
