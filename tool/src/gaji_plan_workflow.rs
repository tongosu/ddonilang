use std::fs;
use std::path::Path;

use crate::artifact_output::write_text_artifact_atomic;

/// Shared owner for the retained local gaji plan adapters.
///
/// The retained CLI is only an adapter: request parsing, product planning, and
/// artifact publication stay in the shared tool authority.
pub fn run_gaji_plan(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let request = fs::read_to_string(input)
        .map_err(|err| format!("E_GAJI_PLAN_READ 가지 추출 요청을 읽을 수 없습니다: {err}"))?;
    let plan = crate::local_project::build_gaji_extraction_plan_json(&request)
        .map_err(|err| err.to_string())?;
    write_or_emit(out, &plan, "E_GAJI_PLAN_WRITE 가지 추출 계획을 쓸 수 없습니다: ")
}

pub fn run_gaji_lifecycle_plan(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let request = fs::read_to_string(input)
        .map_err(|err| format!("E_GAJI_LIFECYCLE_READ 요청을 읽을 수 없습니다: {err}"))?;
    let plan = crate::local_project::build_gaji_lifecycle_plan_json(&request)
        .map_err(|err| err.to_string())?;
    write_or_emit(out, &plan, "E_GAJI_LIFECYCLE_WRITE ")
}

fn write_or_emit(out: Option<&Path>, payload: &str, error_prefix: &str) -> Result<(), String> {
    if let Some(path) = out {
        write_text_artifact_atomic(path, payload).map_err(|err| format!("{error_prefix}{err}"))?;
    } else {
        print!("{payload}");
    }
    Ok(())
}
