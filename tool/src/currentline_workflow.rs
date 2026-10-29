use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentlineWorkflowErrorKind {
    Input,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentlineWorkflowError {
    pub kind: CurrentlineWorkflowErrorKind,
    pub message: String,
}

impl CurrentlineWorkflowError {
    fn input(message: impl Into<String>) -> Self {
        Self {
            kind: CurrentlineWorkflowErrorKind::Input,
            message: message.into(),
        }
    }

    fn runtime(message: impl Into<String>) -> Self {
        Self {
            kind: CurrentlineWorkflowErrorKind::Runtime,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurrentlineWorkflowResult {
    pub summary: JsonValue,
    pub context_json: String,
    pub stdout: String,
}

pub fn execute_currentline_with_supported_profile(
    cell_source: &str,
    source_path: &str,
    context_json: Option<&str>,
    source_profile_identity: &str,
) -> Result<CurrentlineWorkflowResult, CurrentlineWorkflowError> {
    let currentline = ddonirang_lang::apply_currentline_cell_v25(cell_source, context_json)
        .map_err(CurrentlineWorkflowError::input)?;
    let summary = crate::runtime_surface::run_summary_from_supported_source_profile(
        &currentline.project_source,
        source_path,
        None,
        Some(source_profile_identity),
    )
    .map_err(CurrentlineWorkflowError::runtime)?;
    let stdout = currentline_stdout(&summary);
    Ok(CurrentlineWorkflowResult {
        summary,
        context_json: currentline.context_json,
        stdout,
    })
}

fn currentline_stdout(summary: &JsonValue) -> String {
    let Some(lines) = summary
        .get("output_log_texts")
        .and_then(JsonValue::as_array)
    else {
        return String::new();
    };
    let mut output = lines
        .iter()
        .filter_map(JsonValue::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

fn output_sidecar_path(
    target: &Path,
    kind: &str,
    index: usize,
) -> Result<PathBuf, CurrentlineWorkflowError> {
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(CurrentlineWorkflowError::input(format!(
            "E_CURRENTLINE_OUTPUT_PARENT {}",
            parent.display()
        )));
    }
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            CurrentlineWorkflowError::input(format!(
                "E_CURRENTLINE_OUTPUT_PATH {}",
                target.display()
            ))
        })?;
    Ok(parent.join(format!(
        ".{file_name}.ddn-{kind}-{}-{index}",
        std::process::id()
    )))
}

pub fn write_currentline_outputs_atomic(
    outputs: &[(PathBuf, String)],
) -> Result<(), CurrentlineWorkflowError> {
    if outputs.len() == 2 && outputs[0].0 == outputs[1].0 {
        return Err(CurrentlineWorkflowError::input(
            "E_CURRENTLINE_OUTPUT_CONFLICT summary/context targets must differ",
        ));
    }
    struct PreparedOutput {
        target: PathBuf,
        temp: PathBuf,
        backup: PathBuf,
        had_existing: bool,
        installed: bool,
    }
    let mut prepared: Vec<PreparedOutput> = Vec::new();
    for (index, (target, _)) in outputs.iter().enumerate() {
        if target.is_dir() {
            return Err(CurrentlineWorkflowError::input(format!(
                "E_CURRENTLINE_OUTPUT_IS_DIRECTORY {}",
                target.display()
            )));
        }
        let temp = output_sidecar_path(target, "tmp", index)?;
        let backup = output_sidecar_path(target, "backup", index)?;
        if temp.exists() || backup.exists() {
            return Err(CurrentlineWorkflowError::input(format!(
                "E_CURRENTLINE_OUTPUT_RESIDUE {}",
                target.display()
            )));
        }
        prepared.push(PreparedOutput {
            target: target.clone(),
            temp,
            backup,
            had_existing: target.exists(),
            installed: false,
        });
    }
    for (index, (_, text)) in outputs.iter().enumerate() {
        if let Err(error) = fs::write(&prepared[index].temp, text.as_bytes()) {
            for item in &prepared {
                let _ = fs::remove_file(&item.temp);
            }
            return Err(CurrentlineWorkflowError::runtime(format!(
                "E_CURRENTLINE_OUTPUT_WRITE {} {error}",
                prepared[index].temp.display()
            )));
        }
    }
    for index in 0..prepared.len() {
        if prepared[index].had_existing {
            if let Err(error) = fs::rename(&prepared[index].target, &prepared[index].backup) {
                for item in prepared[..index].iter().rev() {
                    if item.had_existing && item.backup.exists() {
                        let _ = fs::rename(&item.backup, &item.target);
                    }
                }
                for item in &prepared {
                    let _ = fs::remove_file(&item.temp);
                }
                return Err(CurrentlineWorkflowError::runtime(format!(
                    "E_CURRENTLINE_OUTPUT_BACKUP {} {error}",
                    prepared[index].target.display()
                )));
            }
        }
    }
    for index in 0..prepared.len() {
        if let Err(error) = fs::rename(&prepared[index].temp, &prepared[index].target) {
            for item in prepared[..index].iter_mut().rev() {
                if item.installed {
                    let _ = fs::remove_file(&item.target);
                    item.installed = false;
                }
            }
            for item in prepared.iter().rev() {
                if item.had_existing && item.backup.exists() {
                    let _ = fs::rename(&item.backup, &item.target);
                }
                let _ = fs::remove_file(&item.temp);
            }
            return Err(CurrentlineWorkflowError::runtime(format!(
                "E_CURRENTLINE_OUTPUT_COMMIT {} {error}",
                prepared[index].target.display()
            )));
        }
        prepared[index].installed = true;
    }
    for item in &prepared {
        if item.backup.exists() {
            let _ = fs::remove_file(&item.backup);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_currentline_executes_and_unknown_profile_fails_closed() {
        let source = "x <- 15.\ny <- 8.\n합 <- x + y.\n합 보여주기.\n";
        let result = execute_currentline_with_supported_profile(
            source,
            "currentline-shared-workflow.ddn",
            None,
            crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        )
        .expect("supported currentline");
        assert_eq!(result.stdout, "23\n");
        assert_eq!(result.summary["output_log_texts"][0], "23");

        let error = execute_currentline_with_supported_profile(
            source,
            "currentline-shared-workflow.ddn",
            None,
            "unknown-profile",
        )
        .expect_err("unknown profile must fail closed");
        assert!(error.message.contains("E_SOURCE_PROFILE_UNSUPPORTED"));
    }
}
