use super::{open_with_diagnostic, registration, PublicCallableOriginalRecordV4, StrictValue};
use crate::ddn_runtime::{default_age_target, DdnProgram, DdnRunner};
use crate::preprocess::ExecutionDiagnostic;
use crate::project_revision_envelope_supply::admit_linked_multilang_project;
use ddonirang_core::{Fixed64, InputSnapshot, InputSource, NuriWorld};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    original_record_json_utf8: String,
    consumer_project_json_utf8: String,
    consumer_source_path: String,
    logic_consumer_source_path: String,
}

fn failure(code: &'static str, message: impl Into<String>) -> ExecutionDiagnostic {
    ExecutionDiagnostic {
        code,
        message: message.into(),
        source_path: String::new(),
        span: None,
        phase: "source_prepare",
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }
}

pub(super) fn run(raw: &str) -> Result<String, ExecutionDiagnostic> {
    if raw.len() > 16_000_000 {
        return Err(failure("E_SOURCE_SAVE_REQUEST_LIMIT", "공동 실행 요청의 크기 한도를 넘었습니다"));
    }
    let invalid_request = |_| failure("E_SOURCE_SAVE_REQUEST_FIELDS", "공동 실행 요청 필드 네 개를 확인하세요");
    let value: StrictValue = serde_json::from_str(raw).map_err(invalid_request)?;
    let input: Request = serde_json::from_value(value.0).map_err(invalid_request)?;
    let mut diagnostic = None;
    let artifact = input.consumer_project_json_utf8.as_bytes();
    let project = open_with_diagnostic(artifact, Some(&mut diagnostic)).map_err(|message| {
        diagnostic.take().unwrap_or_else(|| failure("E_PROJECT_SOURCE_ADMISSION", message))
    })?;
    if input.consumer_source_path == input.logic_consumer_source_path {
        return Err(failure("E_LOGIC_SOURCE_CONTEXT_CONFLICT", "두 등록의 소비 모듈을 구분하세요"));
    }
    let original = PublicCallableOriginalRecordV4::open(&input.original_record_json_utf8)
        .and_then(|record| record.restore())
        .map_err(|message| failure("E_DDL_ORIGINAL_RECORD", message))?;
    let owner = project.authored_identity.modules.iter()
        .find(|module| module.head == input.consumer_source_path)
        .ok_or_else(|| failure("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED", "사용자 DDL의 소비 모듈 원문을 확인하세요"))?;
    let (public, _) = original.activate_for_consumer_selected(
        &project, owner.id.as_str(), &input.consumer_source_path, true, Some(&mut diagnostic),
    ).map_err(|message| {
        diagnostic.take().unwrap_or_else(|| failure("E_DDL_CONSUMER_ACTIVATION", message))
    })?;
    let (logic, _) = registration::logic_dependency::CheckedLogicDependency::for_project_diagnostic(
        &project, &input.logic_consumer_source_path,
        super::source_save::logic_consumer::DDL, super::source_save::logic_consumer::DDL_PATH,
    )?;

    let source = admit_linked_multilang_project(artifact, &public, &logic)?;
    // The registration proof may only use the recorded identities. Any path
    // that needs a new nominal identity must fail before the runner is created.
    let mut draft = registration::prepare_precanon_under_candidate_owner(
        &source, None, &mut |_| Err("E_MULTILANG_JOINT_IDENTITY_ISSUE_FORBIDDEN".into()),
    ).map_err(|message| failure("E_MULTILANG_JOINT_REGISTRATION", message))?;
    draft.attach_logic_dependency(&source, &input.logic_consumer_source_path)
        .map_err(|message| failure("E_MULTILANG_JOINT_REGISTRATION", message))?;
    let mut program = DdnProgram::from_saved_v25_project_with_mode(
        source.sources().clone(), ddonirang_lang::ParseMode::Strict, default_age_target(),
    ).map_err(|message| failure("E_MULTILANG_JOINT_PROGRAM", message))?;
    registration::bind_recorded_logic_source_program(&source, &mut program, &draft)
        .map_err(|message| failure("E_MULTILANG_JOINT_REGISTRATION", message))?;

    let input = InputSnapshot {
        tick_id: 0,
        dt: Fixed64::from_i64(1),
        keys_pressed: 0,
        last_key_name: String::new(),
        pointer_x_i32: 0,
        pointer_y_i32: 0,
        ai_injections: Vec::new(),
        net_events: Vec::new(),
        frame_source: InputSource::Person,
        rng_seed: 0,
    };
    let result = DdnRunner::new(program, source.sources().entry_declaration().name())
        .run_update_diagnostic(&NuriWorld::new(), &input, &Default::default())
        .map_err(|error| {
            let mut diagnostic = crate::runtime_surface::execution_error_diagnostic(error, "");
            if let Some((start, end)) = diagnostic.span.take() {
                if start < end {
                    if let Ok((path, span)) = source.sources().preparation()
                        .original_span(ddonirang_lang::Span::new(start, end))
                    {
                        diagnostic = diagnostic.remap_source(path, (span.start, span.end));
                    }
                }
            }
            diagnostic
        })?;
    Ok(serde_json::json!({
        "output_log_texts": result.display_outputs.iter().map(|row| row.text.as_str()).collect::<Vec<_>>(),
        "source_only": true,
    }).to_string())
}
