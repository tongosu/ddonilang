//! 일반 실행·입력 공급·결과 봉인의 기존 본문. 공개 정체와 처리 순서를 유지한다.
use super::*;

pub(super) fn run_authored_general_input_prefix_with_age(
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
)->Result<GeneralSuccessfulInputPrefix,String> {
    let owned = crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes())?;
    run_general_input_prefix_from_owned(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age)
}
/// Reuse the retained source captures for staging. Re-admitting fresh source
/// would issue new capture ordinals/PESI and lose original execution custody.
pub(super) fn run_general_input_prefix_from_owned(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
)->Result<GeneralSuccessfulInputPrefix,String> {
    run_general_input_prefix_from_owned_with_subject(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age,None,None)
}
pub(super) fn run_general_input_prefix_from_owned_with_subject(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
    replay_source:Option<&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision>,
)->Result<GeneralSuccessfulInputPrefix,String> {
    run_general_input_prefix_with_evidence(owned,artifact,requested_ticks,base_seed,input_snapshots,input_tape,age,
        subject_bootstrap,replay_source.map(GeneralEvidenceSourceV1::Original))
}
pub(super) fn run_general_input_prefix_with_evidence(
    owned:crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    artifact:&str,requested_ticks:u32,base_seed:u64,input_snapshots:Option<&[InputSnapshot]>,
    input_tape:Option<&InputTape>,age:AgeTarget,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
    replay_source:Option<GeneralEvidenceSourceV1<'_>>,
)->Result<GeneralSuccessfulInputPrefix,String> {
    let preparation = owned.sources().preparation();
    let profile = resolve_supported_source_profile(Some("v1-core-v25")).map_err(|error|error.message)?;
    let mut rows = Vec::new();
    let mut execution = None;
    let summary = run_summary_with_general_input_supplier(
        &preparation.source,&preparation.target_path,Some(requested_ticks),None,None,Some(age),
        Some(profile),input_tape,base_seed,owned.sources().entry_declaration().name(),
        None,true,input_snapshots,None,None,Some(artifact),Some(&mut rows),Some(&owned),Some(GeneralExecutionCollector{destination:&mut execution,source:replay_source}),subject_bootstrap,
    ).map_err(|error|error.to_json_value().to_string())?;
    let result = GeneralSuccessfulInputPrefix { owned, rows, base_seed, summary:RuntimeJsonOutput::new(summary),
        execution:execution.ok_or("E_PROJECT_RUNTIME_GENERAL_BOUNDARY: 성공 실행 상태가 없습니다")?,
        construction:GeneralInputConstruction {requested_ticks,age,subject_bootstrap:subject_bootstrap.cloned(),
            snapshots:input_snapshots.map(<[InputSnapshot]>::to_vec),tape:input_tape.cloned()} };
    // Wrong actual seeds and unsupported sources only reject this new save
    // supplier; the existing executable frontdoors retain their semantics.
    result.input_supply().map_err(|error|format!("E_PROJECT_RUNTIME_INPUT_SUPPLY: 실제 입력 저장 결속 실패 ({error:?})"))?;
    Ok(result)
}

pub fn run_summary_from_supported_source_profile(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
) -> Result<JsonValue, String> {
    run_summary_from_supported_source_profile_diagnostic(
        source,
        file_path,
        requested_ticks,
        source_profile_identity,
    )
    .map_err(|err| err.to_json_value().to_string())
}

/// Runs the ordinary shared runtime only after the declared host resources
/// have already been sealed.  The resource receipt is a product result field,
/// not a CLI-local annotation, so native and WASM consumers receive the same
/// identity-bearing payload.
pub fn run_summary_from_supported_source_profile_with_host_resources(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    host_resources: &PreparedHostResources,
) -> Result<JsonValue, String> {
    let mut summary = crate::unit_wire::SnapshotJsonOwner::from_value(run_summary_from_supported_source_profile(
        source,
        file_path,
        requested_ticks,
        source_profile_identity,
    )?);
    attach_host_resource_receipt(summary.value_mut(), host_resources);
    Ok(summary.into_parent_value())
}

pub fn attach_host_resource_receipt(
    summary: &mut JsonValue,
    host_resources: &PreparedHostResources,
) {
    // F5-GA[HOST_SERIALIZE] Transcript/Payload/Receipt use derived Serialize over only String/u64 and flat Vec<Payload>; no custom serializer, unsupported map key, recursive field or external writer can return an error here.
    if let Some(object) = summary.as_object_mut() {
        object.insert(
            "host_resource_transcript".to_string(),
            serde_json::to_value(&host_resources.transcript)
                .expect("host resource transcript serializes"),
        );
        object.insert(
            "host_resource_receipt".to_string(),
            serde_json::to_value(&host_resources.receipt)
                .expect("host resource receipt serializes"),
        );
    }
}

pub fn run_summary_from_supported_source_profile_with_seed(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    rng_base_seed: u64,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        None,
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_project_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    project_asset_registry: Arc<ProjectAssetRegistry>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        Some(project_asset_registry),
        None,
        None,
        Some(profile),
        None,
        0,
    )
}

pub fn run_summary_from_source_with_project_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Arc<ProjectAssetRegistry>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        Some(project_asset_registry),
        None,
        None,
        None,
        None,
        0,
    )
}

pub fn run_summary_from_source_with_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        Some(input_tape),
        0,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_source_with_seed_and_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    rng_base_seed: u64,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        None,
        Some(input_tape),
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        Some(input_tape),
        0,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub fn run_summary_from_supported_source_profile_with_seed_and_input_tape(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    source_profile_identity: Option<&str>,
    rng_base_seed: u64,
    input_tape: &InputTape,
) -> Result<JsonValue, String> {
    let profile = resolve_supported_source_profile(source_profile_identity).map_err(|error| {
        run_summary_diagnostic(error.code, error.message, file_path, "source_profile")
            .to_json_value()
            .to_string()
    })?;
    run_summary_from_source_with_assets_diagnostic(
        source,
        file_path,
        requested_ticks,
        None,
        None,
        None,
        Some(profile),
        Some(input_tape),
        rng_base_seed,
    )
    .map_err(|err| err.to_json_value().to_string())
}

pub(super) fn run_summary_diagnostic(
    code: &'static str,
    message: impl Into<String>,
    file_path: &str,
    phase: &'static str,
) -> ExecutionDiagnostic {
    ExecutionDiagnostic {
        code,
        message: message.into(),
        source_path: file_path.to_string(),
        span: None,
        phase,
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }
}

pub(super) fn work_cost_failure_diagnostic(
    failure: WorkCostFailure,
    file_path: &str,
    phase: &'static str,
) -> ExecutionDiagnostic {
    let receipt = serde_json::to_string(&failure.receipt)
        .unwrap_or_else(|_| "{\"receipt_error\":true}".to_string());
    ExecutionDiagnostic {
        code: failure.code,
        message: format!("{}; work_cost_receipt={receipt}", failure),
        source_path: file_path.to_string(),
        span: None,
        phase,
        cause: None,
        offending_field: None,
        expected: None,
        actual: None,
    }
}

pub(super) fn run_summary_from_source_with_assets_diagnostic(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>,
    age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>,
    input_tape: Option<&InputTape>,
    rng_base_seed: u64,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_assets_diagnostic_with_update_name(
        source,
        file_path,
        requested_ticks,
        project_asset_registry,
        standard_limits,
        age_target,
        source_profile,
        input_tape,
        rng_base_seed,
        DEFAULT_UPDATE_NAME,
        None,
        false,
        None,
        None,
        None,
    )
}

/// The old source frontdoor remains a temporary-identity execution route.
pub(super) fn run_summary_from_source_with_assets_diagnostic_with_update_name(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>, age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>, input_tape: Option<&InputTape>,
    rng_base_seed: u64, update_name: &str,
    live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool, input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>, bogae_entry: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_from_source_with_authored_bogae(
        source, file_path, requested_ticks, project_asset_registry, standard_limits,
        age_target, source_profile, input_tape, rng_base_seed, update_name,
        live_next_mask, named_entry_after_definitions, input_snapshots,
        param_overrides, bogae_entry, None)
}

pub(super) fn run_summary_from_source_with_authored_bogae(
    source: &str,
    file_path: &str,
    requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>,
    age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>,
    input_tape: Option<&InputTape>,
    rng_base_seed: u64,
    update_name: &str,
    live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool,
    input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>,
    bogae_entry: Option<&str>,
    authored_artifact: Option<&str>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    run_summary_with_general_input_supplier(source,file_path,requested_ticks,
        project_asset_registry,standard_limits,age_target,source_profile,input_tape,
        rng_base_seed,update_name,live_next_mask,named_entry_after_definitions,
        input_snapshots,param_overrides,bogae_entry,authored_artifact,None,None,None,None)
}

/// Optional internal supplier shares the actual product loop. Public summaries
/// and old callers are unchanged. A failed run never publishes a partial prefix.
pub(super) fn run_summary_with_general_input_supplier(
    source: &str, file_path: &str, requested_ticks: Option<u32>,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
    standard_limits: Option<StandardResourceLimitsV1>, age_target: Option<AgeTarget>,
    source_profile: Option<SupportedSourceProfile>, input_tape: Option<&InputTape>,
    rng_base_seed: u64, update_name: &str,
    mut live_next_mask: Option<(u32, &mut dyn FnMut(u64) -> Result<InputRecord, String>)>,
    named_entry_after_definitions: bool, input_snapshots: Option<&[InputSnapshot]>,
    param_overrides: Option<&BTreeMap<String, Value>>, bogae_entry: Option<&str>,
    authored_artifact: Option<&str>,
    successful_inputs: Option<&mut Vec<crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow>>,
    general_saved_owner: Option<&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
    completed_execution: Option<GeneralExecutionCollector<'_>>,
    subject_bootstrap:Option<&crate::unit_wire::subject_execution::SubjectExecutionStage>,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let mut staged_inputs = Vec::new();
    #[cfg(test)]
    RUN_SUMMARY_DIAGNOSTIC_INVOCATIONS.with(|count| count.set(count.get() + 1));

    if ddonirang_lang::has_model_surface(source) {
        if authored_artifact.is_some() {
            return Err(run_summary_diagnostic("E_AUTHORED_BOGAE_MODEL_UNSUPPORTED",
                "모델 원문의 영구 보개 정체 연결은 아직 지원하지 않습니다", file_path, "authored_bogae"));
        }
        if live_next_mask.is_some() || input_snapshots.is_some() {
            return Err(run_summary_diagnostic("E_SHARED_RUNTIME_OPTION_UNSUPPORTED", "모델 실행 경로는 실시간 제어입력을 지원하지 않습니다", file_path, "live_input"));
        }
        if rng_base_seed != 0 {
            return Err(run_summary_diagnostic(
                "E_MODEL_SEED_UNSUPPORTED",
                "model runtime에는 explicit seed 계약이 없습니다",
                file_path,
                "model_execution",
            ));
        }
        validate_model_oneshot_ticks(requested_ticks).map_err(|message| {
            run_summary_diagnostic("E_MODEL_TICK_POLICY", message, file_path, "model_execution")
        })?;
        return model_run_summary_from_source(source, file_path, project_asset_registry);
    }
    let source_profile_identity = source_profile
        .map(|_| V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY)
        .unwrap_or("strict");
    let effective_age_target = age_target.unwrap_or_else(crate::ddn_runtime::default_age_target);
    let (prepared, mut program) = if let Some(owned) = general_saved_owner {
        let mut program = DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ParseMode::Strict,effective_age_target,
        ).map_err(|message|run_summary_diagnostic("E_PROJECT_RUNTIME_SOURCE",message,file_path,"saved_source"))?;
        program.bind_registered_project_context(owned)
            .map_err(|message|run_summary_diagnostic("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED",message,file_path,"saved_source"))?;
        let prepared = prepare_supported_source_for_execution_with_program(source,file_path,&program)?;
        (prepared,program)
    } else { match source_profile {
        Some(profile) => {
            let program = DdnProgram::from_supported_source_profile_with_mode_and_age_target(
                source,
                file_path,
                ParseMode::Strict,
                profile,
                effective_age_target,
            )
            .map_err(|message| {
                if let Some(diagnostic) = supported_source_parse_diagnostic(source, file_path, &message) {
                    return diagnostic;
                }
                let code = if message.contains(E_V1_CORE_RECURSION_UNSUPPORTED) {
                    E_V1_CORE_RECURSION_UNSUPPORTED
                } else if message.contains("E_V25_DEFINITION_MIGRATION_REQUIRED") {
                    "E_V25_DEFINITION_MIGRATION_REQUIRED"
                } else if message.contains("E_RUNTIME_TYPE_MISMATCH") {
                    "E_RUNTIME_TYPE_MISMATCH"
                } else if message.contains("E_AGE_NOT_AVAILABLE") {
                    "E_AGE_NOT_AVAILABLE"
                } else {
                    "E_PARSE"
                };
                run_summary_diagnostic(code, message, file_path, "execution_parse")
            })?;
            let prepared =
                prepare_supported_source_for_execution_with_program(source, file_path, &program)?;
            (prepared, program)
        }
        None => {
            let prepared = prepare_source_for_execution_with_path(source, file_path)?;
            let program = program_from_legacy_source_with_age_target(
                source,
                file_path,
                ParseMode::Strict,
                effective_age_target,
            )
            .map_err(|message| {
                let code = if message.contains("E_AGE_NOT_AVAILABLE") {
                    "E_AGE_NOT_AVAILABLE"
                } else {
                    "E_PARSE"
                };
                run_summary_diagnostic(code, message, file_path, "execution_parse")
            })?;
            (prepared, program)
        }
    }};
    crate::source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|message| {
            run_summary_diagnostic("E_STATIC_VALIDATION", message, file_path, "static_validation")
        })?;
    if general_saved_owner.is_none(){
        program.prepare_temporary_source_execution().map_err(|message|
            run_summary_diagnostic("E_VC04_TEMPORARY_SOURCE",message,file_path,"source_registration"))?;
    }
    let configured_madi = program.configured_madi().unwrap_or(0);
    let execution_mode_decision = prepared.execution_mode_decision;
    let parse_warnings = program.parse_warnings().to_vec();
    let (input_masks, sealed_input_tape_sha256) = match input_tape {
        Some(tape) => {
            if tape.madi_hz == 0 {
                return Err(run_summary_diagnostic(
                    "E_SAM_BAD_MADI_HZ",
                    "madi-hz는 0이 될 수 없습니다",
                    file_path,
                    "input_tape",
                ));
            }
            if tape.records.is_empty() {
                return Err(run_summary_diagnostic(
                    "E_SAM_RECORD_COUNT_MISMATCH",
                    "record_count=0",
                    file_path,
                    "input_tape",
                ));
            }
            if let Some(ticks) = requested_ticks {
                if usize::try_from(ticks).ok() != Some(tape.records.len()) {
                    return Err(run_summary_diagnostic(
                        "E_SAM_RECORD_COUNT_MISMATCH",
                        format!("record_count={} ticks={ticks}", tape.records.len()),
                        file_path,
                        "input_tape",
                    ));
                }
            }
            let mut masks = Vec::with_capacity(tape.records.len());
            for (index, record) in tape.records.iter().enumerate() {
                if record.madi != index as u32 {
                    return Err(run_summary_diagnostic(
                        "E_SAM_RECORD_ORDER_MISMATCH",
                        format!("record.madi={} idx={index}", record.madi),
                        file_path,
                        "input_tape",
                    ));
                }
                let mask = mask_from_bytes(&record.held_mask).map_err(|message| {
                    run_summary_diagnostic("E_SAM_TAPE", message, file_path, "input_tape")
                })?;
                masks.push(tape_mask_to_runtime_keys(mask));
            }
            let bytes = encode_input_tape(tape).map_err(|message| {
                run_summary_diagnostic("E_SAM_TAPE", message, file_path, "input_tape")
            })?;
            (masks, Some(sha256_hex(&bytes)))
        }
        None => (Vec::new(), None),
    };
    let effective_ticks = if input_tape.is_some() {
        input_masks.len() as u64
    } else {
        requested_ticks
            .map(u64::from)
            .unwrap_or_else(|| configured_madi.max(1))
            .max(1)
    };

    let sealed_dataset_assets = match project_asset_registry.as_deref() {
        Some(registry) => {
            serde_json::to_value(registry.sealed_dataset_identities().map_err(|message| {
                run_summary_diagnostic("E_DATASET_IDENTITY", message, file_path, "dataset_identity")
            })?)
            .map_err(|err| {
                run_summary_diagnostic(
                    "E_DATASET_IDENTITY_SERIALIZE",
                    err.to_string(),
                    file_path,
                    "dataset_identity",
                )
            })?
        }
        None => JsonValue::Array(Vec::new()),
    };
    let sealed_dataset_identity_sha256 = sha256_hex(
        serde_json::to_string(&sealed_dataset_assets)
            .map_err(|err| {
                run_summary_diagnostic(
                    "E_DATASET_IDENTITY_SERIALIZE",
                    err.to_string(),
                    file_path,
                    "dataset_identity",
                )
            })?
            .as_bytes(),
    );
    let mut runner = DdnRunner::new(program, update_name).with_bogae_activation();
    if let Some(stage)=subject_bootstrap {runner.bind_subject_bootstrap(stage.clone());}
    if let Some(owned) = general_saved_owner {
        runner.configure_project_entry(owned.sources().entry_declaration().name()).map_err(|message|
            run_summary_diagnostic("E_PROJECT_RUNTIME_ENTRY",message,file_path,"saved_source"))?;
        let project = owned.project();
        if project.schema == crate::authored_project::SCHEMA_V2
            || project.authored_identity.modules.iter().any(|module|!module.compositions.is_empty()) {
            runner.configure_authored_bogae(authored_artifact.ok_or_else(||run_summary_diagnostic(
                "E_PROJECT_RUNTIME_SOURCE", "작품 작성 자료가 없습니다",file_path,"saved_source"))?).map_err(|message|
                run_summary_diagnostic("E_AUTHORED_BOGAE_ADMISSION",message,file_path,"authored_bogae"))?;
        }
    } else if let Some(artifact) = authored_artifact {
        runner.configure_authored_bogae(artifact).map_err(|message|
            run_summary_diagnostic("E_AUTHORED_BOGAE_ADMISSION", message, file_path, "authored_bogae"))?;
    }
    if let Some(entry) = bogae_entry {
        runner.configure_bogae_entry(entry).map_err(|message|
            run_summary_diagnostic("E_BOGAE_ENTRY_UNRESOLVED", message, file_path, "bogae_entry"))?;
    }
    if named_entry_after_definitions {
        runner = runner.with_named_entry_after_definitions();
    }
    if let Some(limits) = standard_limits {
        runner = runner.with_standard_resource_limits(limits);
    }
    let mut sealed_input_bytes = input_tape
        .and_then(|tape| encode_input_tape(tape).ok())
        .map(|bytes| source.len().saturating_add(bytes.len()))
        .unwrap_or(source.len());
    if let Some(inputs) = input_snapshots {
        let bytes = json!(inputs.iter().map(crate::input_receipt::snapshot_value)
            .collect::<Vec<_>>()).to_string().len();
        sealed_input_bytes = sealed_input_bytes.checked_add(bytes).ok_or_else(||
            run_summary_diagnostic("E_INPUT_RECEIPT_SIZE", "입력 기록 크기가 표현 범위를 넘었습니다",
                file_path, "resource_preflight"))?;
    }
    runner
        .set_sealed_input_bytes(sealed_input_bytes)
        .map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight")
        })?;
    if let Some(registry) = project_asset_registry {
        runner = runner.with_project_asset_registry(registry);
    }
    let mut world = DetNuri::new();
    let mut defaults = HashMap::new();
    seed_bogae_defaults(&mut defaults);
    let mut tick_id = 0u64;
    let input_dt = crate::input_receipt::alpha_host_dt();
    let mut last_patch = Patch::default();
    let mut last_input = serialize_input_state(0, "", 0, 0, input_dt, rng_base_seed, rng_base_seed);
    let source_identity = format!("sha256:{}", sha256_hex(source.as_bytes()));
    let revision_identity = format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "{}\0{}",
                prepared.canonical_ddn, sealed_dataset_identity_sha256
            )
            .as_bytes()
        )
    );
    if let Some(inputs) = input_snapshots {
        if input_tape.is_some() || live_next_mask.is_some() || inputs.len() != effective_ticks as usize
            || inputs.iter().enumerate().any(|(index, input)| input.tick_id != index as u64) {
            return Err(run_summary_diagnostic("E_INPUT_RECEIPT_SEQUENCE_INVALID",
                "입력 기록의 마디 수·순서 또는 입력 경로가 다릅니다", file_path, "input_receipt"));
        }
    }
    let input_record_sha256 = input_snapshots.map(crate::input_receipt::record_sha256);
    let mut input_identity_suffix = sealed_input_tape_sha256
        .as_deref()
        .map(|identity| format!("\0input_tape_sha256={identity}"))
        .unwrap_or_default();
    if let Some(identity) = &input_record_sha256 {
        input_identity_suffix.push_str(&format!("\0input_record_sha256={identity}"));
    }
    let seed_identity_suffix = if rng_base_seed == 0 {
        String::new()
    } else {
        format!("\0rng_base_seed={rng_base_seed}")
    };
    let run_identity_material = |input_identity_suffix: &str| match age_target {
        Some(age_target) => format!(
            "{}\0{}\0{}\0{}\0age_target={}{}{}",
            file_path,
            source_identity,
            revision_identity,
            effective_ticks,
            age_target.label(),
            input_identity_suffix,
            seed_identity_suffix
        ),
        None => format!(
            "{}\0{}\0{}\0{}{}{}",
            file_path,
            source_identity,
            revision_identity,
            effective_ticks,
            input_identity_suffix,
            seed_identity_suffix
        ),
    };
    let mut run_id = format!("run:{}", sha256_hex(run_identity_material(&input_identity_suffix).as_bytes()));
    let mut live_records = Vec::new();
    let mut input_receipts = Vec::new();
    let mut last_bogae_projection = None;
    let mut display_outputs = Vec::new();
    let mut observation_frames = preflight_observation_frames(&runner, effective_ticks as usize)
        .map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight")
        })?;

    // Capture only here, before the first actual input, from the original
    // source-only owner. A legacy/cold prefix never manufactures this proof.
    let mut result_collector=match completed_execution.as_ref().and_then(|collector|collector.source){
        Some(original)=>Some(GeneralResultCollectorV1::from_source(general_saved_owner.ok_or_else(||
            run_summary_diagnostic("E_GENERAL_REPLAY_SEED_SOURCE","원 실행의 원문 소유자가 없습니다",file_path,"result_evidence"))?,
            &runner,original,rng_base_seed,effective_age_target)
            .map_err(|message|run_summary_diagnostic("E_GENERAL_REPLAY_SEED",message,file_path,"result_evidence"))?),
        None=>None,
    };
    for tick_index in 0..effective_ticks {
        // F5-GA[GENERAL_INPUT] Explicit snapshots were checked to have exactly effective_ticks entries before this loop; the borrowed slice remains immutable.
        let tick_seed = rng_base_seed ^ tick_id;
        let keys_pressed = match live_next_mask.as_mut() {
            Some((_, next_mask)) => {
                let record = next_mask(tick_id).map_err(|message| {
                    let code = match message.split_whitespace().next() {
                        Some("E_SAM_LIVE_QUEUE_OVERFLOW") => "E_SAM_LIVE_QUEUE_OVERFLOW",
                        Some("E_SAM_LIVE_SEQUENCE_OVERFLOW") => "E_SAM_LIVE_SEQUENCE_OVERFLOW",
                        Some("E_SAM_LIVE_INPUT_POLL") => "E_SAM_LIVE_INPUT_POLL",
                        Some("E_SAM_LIVE_INPUT_READ") => "E_SAM_LIVE_INPUT_READ",
                        Some("E_SAM_LIVE_RAW_RESTORE") => "E_SAM_LIVE_RAW_RESTORE",
                        _ => "E_SAM_LIVE_INPUT",
                    };
                    run_summary_diagnostic(code, message, file_path, "live_input")
                })?;
                if u64::from(record.madi) != tick_id {
                    return Err(run_summary_diagnostic("E_SAM_RECORD_ORDER_MISMATCH", "실시간 입력의 마디 순서가 다릅니다", file_path, "live_input"));
                }
                let mask = mask_from_bytes(&record.held_mask).map_err(|message| run_summary_diagnostic("E_SAM_TAPE", message, file_path, "live_input"))?;
                live_records.push(record);
                tape_mask_to_runtime_keys(mask)
            },
            None => input_masks.get(tick_index as usize).copied().unwrap_or(0),
        };
        let input = input_snapshots.map(|inputs| inputs[tick_index as usize].clone()).unwrap_or_else(|| InputSnapshot {
            tick_id,
            dt: input_dt,
            keys_pressed,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: tick_seed,
        });
        let control_edges = if live_next_mask.is_some() {
            live_records.last().map(InputRecord::runtime_edges)
        } else {
            input_tape.filter(|tape| tape.version == 2)
                .and_then(|tape| tape.records.get(tick_index as usize))
                .map(InputRecord::runtime_edges)
        };
        let output = runner
            .run_update_diagnostic_with_control_edges(world.world(), &input, &defaults, control_edges, param_overrides)
            .map_err(|error| execution_error_diagnostic(error, file_path))?;
        let actual_receipt=crate::input_receipt::receipt(&input,&output.input_reads);
        let prepared_result=match result_collector.as_mut(){
            Some(collector)=>Some(collector.prepare(output.result_emissions,tick_id,&actual_receipt)
                .map_err(|message|run_summary_diagnostic("E_GENERAL_RESULT_OUTPUT",message,file_path,"result_evidence"))?),
            None=>None,
        };
        if input_tape.is_some_and(|tape| tape.version == 1) {
            crate::input_receipt::validate_tape_v1_reads(&output.input_reads)
                .map_err(|message| run_summary_diagnostic(
                    "E_INPUT_TAPE_V1_UNRECORDED_READ", message, file_path, "input_tape"))?;
        }
        let public_observations = public_observation_events_json(&output.public_observations)
            .map_err(|message| {
                run_summary_diagnostic(
                    "E_OBSERVATION_SERIALIZE",
                    message,
                    file_path,
                    "observation_projection",
                )
            })?;
        let mut sink = ddonirang_core::signals::VecSignalSink::default();
        world.apply_patch(&output.patch, input.tick_id, &mut sink);
        display_outputs.extend(output.display_outputs);
        if !display_outputs.is_empty() {
            crate::output_display::retain_current(&mut display_outputs, &serialize_world_resources(world.world()));
        }
        let input_json = serialize_input_snapshot(&input, Some(rng_base_seed));
        input_receipts.push(actual_receipt);
        if successful_inputs.is_some() {
            staged_inputs.try_reserve(1).map_err(|_|run_summary_diagnostic(
                "E_PROJECT_RUNTIME_INPUT_SUPPLY", "실제 처리 입력을 보존할 공간이 없습니다",
                file_path, "input_supply"))?;
            staged_inputs.push(crate::unit_wire::resumable_complete_state::GeneralProcessedInputRow {
                input: input.clone(), edges: control_edges,
            });
        }
        observation_frames.push(json!({
            "run_id": run_id,
            "source_identity": source_identity,
            "revision_identity": revision_identity,
            "committed_madi": input.tick_id,
            "state_hash": current_state_hash(world.world()),
            "input_snapshot": input_json,
            "public_observations": public_observations,
        }));
        last_patch = output.patch;
        last_bogae_projection = output.bogae_projection;
        last_input = serialize_input_snapshot(&input, Some(rng_base_seed));
        if let (Some(collector),Some(prepared))=(result_collector.as_mut(),prepared_result){
            collector.commit(prepared).map_err(|message|run_summary_diagnostic("E_GENERAL_RESULT_OUTPUT",message,file_path,"result_evidence"))?;
        }
        tick_id = tick_id.saturating_add(1);
    }

    if let Some((madi_hz, _)) = live_next_mask {
        let bytes = encode_input_tape(&InputTape { version: 2, madi_hz, records: live_records })
            .map_err(|message| run_summary_diagnostic("E_SAM_TAPE", message, file_path, "live_input"))?;
        let sealed_bytes = source.len().checked_add(bytes.len()).ok_or_else(||
            run_summary_diagnostic("E_SAM_TAPE", "입력 크기가 표현 범위를 넘었습니다", file_path, "live_input"))?;
        runner.set_sealed_input_bytes(sealed_bytes).map_err(|failure|
            work_cost_failure_diagnostic(failure, file_path, "resource_preflight"))?;
        // All frames are private candidates until input sealing and host restoration succeed.
        run_id = format!("run:{}", sha256_hex(run_identity_material(&format!("\0input_tape_sha256={}", sha256_hex(&bytes))).as_bytes()));
        for frame in &mut observation_frames {
            frame["run_id"] = json!(run_id);
        }
    }
    if let Some(projection)=last_bogae_projection.as_mut() {
        crate::bogae_time::resolve_result_tables(projection,&observation_frames,&source_identity,&revision_identity)
            .map_err(|message|run_summary_diagnostic("E_BOGAE_TIME_PROJECTION",message,file_path,"observation_projection"))?;
    }
    let mut state = crate::unit_wire::SnapshotJsonOwner::from_value(
        build_engine_state_json(&world, tick_id, last_input, &last_patch));
    crate::bogae_projection::attach(state.value_mut(), last_bogae_projection.as_ref());
    crate::output_display::attach(state.value_mut(), &display_outputs);
    let trace_bundle = build_public_observation_trace(&json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": observation_frames,
    }))
    .map_err(|message| {
        run_summary_diagnostic(
            "E_OBSERVATION_TRACE",
            message,
            file_path,
            "observation_projection",
        )
    })?;
    let observation_trace = trace_bundle
        .get("trace")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_trace_hash = trace_bundle
        .get("trace_hash")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let observation_table_projection =
        build_public_observation_table_projection(&observation_trace, &run_id).map_err(
            |message| {
                run_summary_diagnostic(
                    "E_OBSERVATION_TABLE_PROJECTION",
                    message,
                    file_path,
                    "observation_projection",
                )
            },
        )?;
    let observation_columns = observation_trace
        .get("columns")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    let output_rows = observation_trace
        .get("samples")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    if let Some(state_map) = state.value_mut().as_object_mut() {
        state_map.insert(
            "public_observation_trace".to_string(),
            observation_trace.clone(),
        );
        state_map.insert(
            "public_observation_trace_hash".to_string(),
            observation_trace_hash.clone(),
        );
        state_map.insert(
            "public_observation_table_projection".to_string(),
            observation_table_projection.clone(),
        );
        state_map.insert(
            "observation_columns".to_string(),
            observation_columns.clone(),
        );
        state_map.insert("output_rows".to_string(), output_rows.clone());
        if let Some(nested_state) = state_map
            .get_mut("state")
            .and_then(JsonValue::as_object_mut)
        {
            nested_state.insert(
                "snapshot_v2".to_string(),
                runtime_state_snapshot(&world, observation_frames.clone(), tick_id),
            );
        }
    }
    let state_hash = state
        .get("state_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let view_hash = state
        .get("view_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let primary_view_decision = crate::primary_view::decide(
        state.get("view_meta").unwrap_or(&JsonValue::Null),
        &sha256_hex(source.as_bytes()),
        &sha256_hex(prepared.canonical_ddn.as_bytes()),
        &state_hash,
        &view_hash,
    );
    if let Some(state_map) = state.value_mut().as_object_mut() {
        state_map.insert(
            "primary_view_decision".to_string(),
            primary_view_decision.clone(),
        );
    }
    let final_row = index_row_from_state(&state);
    let output_log_texts = output_log_texts_from_state_json(&state);
    let parse_warnings_json = parse_warnings
        .iter()
        .map(|warning| {
            json!({
                "code": warning.code,
                "message": warning.message,
                "span": {
                    "start": warning.span_start,
                    "end": warning.span_end,
                }
            })
        })
        .collect::<Vec<_>>();

    let mut execution_envelope = build_execution_envelope(
        source,
        &prepared,
        configured_madi,
        execution_mode_decision,
        requested_ticks,
        effective_ticks,
        input_dt,
        rng_base_seed,
    );
    if let Some(envelope) = execution_envelope.as_object_mut() {
        envelope.insert("input_record_sha256".into(), json!(input_record_sha256));
        if let Some(inputs) = input_snapshots {
            envelope.insert("input_snapshots".into(), json!(inputs.iter()
                .map(crate::input_receipt::snapshot_value).collect::<Vec<_>>()));
            envelope.insert("input_snapshot".into(), if inputs.len() == 1 {
                // F5-GA[GENERAL_FIRST] This branch proves index0; the dt branch below separately requires first().is_some_and before reading the same first entry.
                serialize_input_snapshot(&inputs[0], None)
            } else { JsonValue::Null });
            envelope.insert("dt".into(), if inputs.first().is_some_and(|first|
                inputs.iter().all(|input| input.dt == first.dt)) {
                json!(crate::input_receipt::exact_dt_text(inputs[0].dt))
            } else { JsonValue::Null });
        }
        envelope.insert(
            "sealed_dataset_identity_sha256".to_string(),
            JsonValue::String(sealed_dataset_identity_sha256.clone()),
        );
        envelope.insert(
            "sealed_dataset_assets".to_string(),
            sealed_dataset_assets.clone(),
        );
        if let Some(age_target) = age_target {
            envelope.insert(
                "age_target_identity".to_string(),
                JsonValue::String(age_target.label().to_string()),
            );
        }
    }
    runner
        .set_replay_capsule_bytes(0)
        .map_err(|failure| work_cost_failure_diagnostic(failure, file_path, "resource_commit"))?;
    let mut summary = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
        "schema": "ddn.ddonirang_tool.shared_run_summary.v1",
        "runtime_schema": ENGINE_RESPONSE_SCHEMA,
        "update_name": update_name,
        "source_path": file_path,
        "source_profile_identity": source_profile_identity,
        "canonical_ddn": prepared.canonical_ddn,
        "preprocessed_source": prepared.execution_source,
        "configured_ticks": configured_madi,
        "effective_ticks": effective_ticks,
        "ticks_run": effective_ticks,
        "parse_warnings": parse_warnings_json,
        "state_hash": state.get("state_hash").cloned().unwrap_or(JsonValue::String(String::new())),
        "view_hash": state.get("view_hash").cloned().unwrap_or(JsonValue::String(String::new())),
        "primary_view_decision": primary_view_decision,
        "final_row": final_row,
        "resources": JsonValue::Null,
        "channels": JsonValue::Null,
        "row": JsonValue::Null,
        "observation_columns": JsonValue::Null,
        "public_observation_trace": JsonValue::Null,
        "public_observation_trace_hash": JsonValue::Null,
        "public_observation_table_projection": JsonValue::Null,
        "output_log_texts": JsonValue::Null,
        "output_rows": JsonValue::Null,
        "engine_state": JsonValue::Null,
        "execution_envelope": execution_envelope,
        "input_receipts": input_receipts,
        "input_record_sha256": input_record_sha256,
        "sealed_dataset_identity_sha256": sealed_dataset_identity_sha256,
        "sealed_dataset_assets": sealed_dataset_assets,
        "parity_surface": {
            "domain": "ddonirang_tool_shared_runtime",
            "state_hash_formula_change": false,
            "view_prefix_exclusion": DEFAULT_VIEW_PREFIXES
        },
        "work_cost_receipt": runner.work_cost_receipt(),
        "meta_lines": prepared.meta_lines
    }));
    // engine_state는 이미 완성된 공개 JSON이다. 동일 슬롯으로 옮겨 큰 상태의
    // json! to_value 재변환을 피한다. 공개 사본·필드·bytes는 바꾸지 않는다.
    for key in ["resources", "channels", "row"] {
        summary.value_mut()[key] = state.get(key).map(|value|
            crate::unit_wire::SnapshotJsonOwner::clone_from_value(value).into_parent_value()).unwrap_or_else(|| {
            if key == "resources" { JsonValue::Object(Map::new()) }
            else { JsonValue::Array(Vec::new()) }
        });
    }
    for (key, value) in [
        ("observation_columns", observation_columns),
        ("public_observation_trace", observation_trace),
        ("public_observation_trace_hash", observation_trace_hash),
        ("public_observation_table_projection", observation_table_projection),
        ("output_rows", output_rows),
    ] {
        summary.value_mut()[key] = value;
    }
    summary.value_mut()["output_log_texts"] = JsonValue::Array(
        output_log_texts.into_iter().map(JsonValue::String).collect());
    summary.value_mut()["engine_state"] = state.into_parent_value();
    crate::bogae_projection::attach(summary.value_mut(), last_bogae_projection.as_ref());
    crate::output_display::attach(summary.value_mut(), &display_outputs);
    if let Some(age_target) = age_target {
        // F5-GA[GENERAL_SUMMARY] The summary root was constructed as an Object; all intermediate setters and both attach helpers only insert fields, not replace this root.
        summary.value_mut().as_object_mut().expect("summary object").insert(
            "age_target_identity".to_string(),
            JsonValue::String(age_target.label().to_string()),
        );
    }
    let summary = finalize_run_summary_result_artifact(summary, &mut runner, file_path)?;
    if let Some(destination) = successful_inputs { *destination = staged_inputs; }
    if let Some(collector) = completed_execution { *collector.destination = Some(GeneralCompletedExecution { runner, world, next_tick:tick_id,result:result_collector,saved_runtime:None }); }
    Ok(summary)
}

pub(super) fn finalize_run_summary_result_artifact(
    mut summary: crate::unit_wire::SnapshotJsonOwner,
    runner: &mut DdnRunner,
    file_path: &str,
) -> Result<JsonValue, ExecutionDiagnostic> {
    let serialize_error = |error: serde_json::Error| {
        run_summary_diagnostic(
            "E_RESULT_ARTIFACT_SERIALIZE",
            error.to_string(),
            file_path,
            "resource_commit",
        )
    };
    let initial_bytes = serialized_json_byte_count(&summary).map_err(&serialize_error)?;
    let initial_receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"])
        // F5-GA[GENERAL_RECEIPT_READ] Both receipt size reads borrow serde_json Value via shared Index; wrong/missing shape yields Null rather than IndexMut panic.
        .map_err(&serialize_error)?;
    // 조립 완료 뒤에는 영수증 값만 바뀐다. 키·구분자·다른 공개 필드의
    // UTF-8 바이트는 고정이므로 전체를 다시 순회하지 않고 분해한 크기를 쓴다.
    let outside_receipt_bytes = initial_bytes.checked_sub(initial_receipt_bytes)
        .ok_or_else(|| serialize_error(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other, "JSON byte count overflow"))))?;
    for _ in 0..16 {
        let receipt_bytes = serialized_json_byte_count(&summary["work_cost_receipt"])
            .map_err(&serialize_error)?;
        let bytes = outside_receipt_bytes.checked_add(receipt_bytes)
            .ok_or_else(|| serialize_error(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::Other, "JSON byte count overflow"))))?;
        if runner.work_cost_receipt().counters.result_artifact_bytes == bytes as u64 {
            return Ok(summary.into_parent_value());
        }
        runner.set_result_artifact_bytes(bytes).map_err(|failure| {
            work_cost_failure_diagnostic(failure, file_path, "resource_commit")
        })?;
        summary.value_mut()["work_cost_receipt"] =
            serde_json::to_value(runner.work_cost_receipt()).map_err(|error| {
                run_summary_diagnostic(
                    "E_RESULT_ARTIFACT_SERIALIZE",
                    error.to_string(),
                    file_path,
                    "resource_commit",
                )
            })?;
    }
    Err(run_summary_diagnostic(
        "E_RESULT_ARTIFACT_SIZE_FIXED_POINT",
        "결과 artifact byte 크기와 WorkCostV1 영수증이 수렴하지 않았습니다",
        file_path,
        "resource_commit",
    ))
}

