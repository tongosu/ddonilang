//! Common B1 provider and existing transport DTO glue. Public execution is
//! routed explicitly after the fixed physical verdict; tests use the same
//! product parser/registry/capture/registration/Native owner, never a translator.
use super::{Source,Probe,Prepare,Open,FirstCallableRun,FirstCallableNative,bytes};
use crate::authored_project as authored;
use authored::registration::logic_dependency::CheckedLogicDependency as Dependency;
pub(in crate::authored_project) const DDL:&str=include_str!("logic_r1.ddl");
pub(in crate::authored_project) const DDL_PATH:&str="論理.ddl";
fn request<T:serde::de::DeserializeOwned>(raw:&str)->Result<T,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let value:authored::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    serde_json::from_value(value.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS".into())
}
pub(crate) fn selected(project:&authored::ProjectArtifact)->Result<Option<Dependency>,String>{
    selected_context(project,None).map(|row|row.map(|(record,_)|record))
}
fn selected_context(project:&authored::ProjectArtifact,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)
    ->Result<Option<(Dependency,std::sync::Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>)>,String>{
    if project.source_profile!=ddonirang_lang::ddl::J1_SOURCE_PROFILE{return Ok(None);}
    let syntax=ddonirang_lang::ddl::CheckedJ1Profile::resolve(&project.source_profile).map_err(|e|e.to_string())?;
    let path=&project.source_snapshot.entry_path;
    let file=project.source_snapshot.files.iter().find(|file|&file.path==path).ok_or("E_LOGIC_SOURCE_REQUIRED")?;
    let known=project.authored_identity.modules.iter().map(|m|m.name.clone()).chain(std::iter::once("logic".into()));
    let mut directive_context=ddonirang_lang::ModuleDirectiveContext::default().with_known_imports(known);
    // Selection must not reject existing non-builtin exports merely because
    // its inspection context omitted the actual owner's public targets.
    // These optional facts confer no builtin authority; selected B1 is still
    // fully rechecked below and cannot export through this first scope.
    let owners=project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect::<Vec<_>>();
    if owners.len()==1{
        if let Ok(facts)=authored::public_surface::from_project(project,owners[0].id.as_str()){
            directive_context=directive_context.with_public_definitions(facts.public_targets());
        }
    }
    let document=ddonirang_lang::module_directive::parse_module_directive_prefix_with_j1_profile(&file.text,path,
        &directive_context,syntax).map_err(|e|authored::registration::logic_dependency::keep_parse_failure(e,path,&mut diagnostic))?;
    let Some(import)=document.imports else{return Ok(None)};
    if !import.entries.iter().any(|row|row.target=="logic"){return Ok(None);}
    if import.translation.is_none(){
        if project.authored_identity.modules.iter().any(|m|m.name=="logic"){return Ok(None);}
        let message="E_DDL_ACTIVATION_REQUIRED: 論理 builtin에는 원문의 명시 翻訳 결속이 필요합니다".to_string();
        if let Some(output)=diagnostic.as_deref_mut(){if output.is_none(){*output=Some(crate::preprocess::ExecutionDiagnostic{
            code:"E_DDL_ACTIVATION_REQUIRED",message:message.clone(),source_path:path.clone(),span:Some((import.span.start,import.span.end)),
            phase:"lang_parse",cause:None,offending_field:None,expected:None,actual:None});}}
        return Err(message);
    }
    match Dependency::for_project_diagnostic(project,path,DDL,DDL_PATH){
        Ok(value)=>Ok(Some(value)),Err(error)=>{
            let message=error.message.clone();if let Some(output)=diagnostic.as_deref_mut(){if output.is_none(){*output=Some(error);}}Err(message)
        }
    }
}
// Existing CLI operations use this checked path after the fixed B1 material
// and production profile physical verdict (2026-11-01, blocking 0). This is
// not product approval: each request verifies source/Bool proof, fixed profile,
// owned pins, registration and Native readback. Generic readers stay closed.
pub(crate) fn probe(raw:&str)->Result<String,String>{probe_selected(raw,None)}
fn probe_selected(raw:&str,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    let input:Probe=request(raw)?;let project=authored::open_with_diagnostic(input.artifact_json_utf8.as_bytes(),diagnostic.as_deref_mut())?;
    if selected_context(&project,diagnostic.as_deref_mut())?.is_some(){return Ok("true".into());}
    super::requirements_diagnostic(raw,diagnostic)
}
pub(super) fn prepare_is_selected(raw:&str)->Result<bool,String>{
    let input:Prepare=request(raw)?;bytes(&input.expected_parent_hex,true)?;bytes(&input.revision_hex,false)?;let project=authored::open(input.artifact_json_utf8.as_bytes())?;
    Ok(selected(&project)?.is_some())
}
pub(super) fn artifact_is_selected(raw:&str)->Result<bool,String>{
    // Selection only, never authority. A selected candidate is strictly read
    // by Source before any private run/save; the public path only refuses it.
    if raw.len()>8_000_000{return Err("E_SOURCE_ARTIFACT_LIMIT".into());}
    let value:authored::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_ARTIFACT_FIELDS")?;
    let Some(payload)=value.0.get("project_payload_utf8").and_then(serde_json::Value::as_str)else{return Ok(false)};
    recorded_payload_is_selected(payload.as_bytes())
}
// Codec selection stays inside the owner's strict decoder; admission still
// verifies the complete material, source ownership and envelope.
pub(crate) fn recorded_payload_is_selected(raw:&[u8])->Result<bool,String>{
    let payload:authored::StrictValue=serde_json::from_slice(raw).map_err(|_|"E_PROJECT_RUNTIME_PAYLOAD_FIELDS")?;
    Ok(payload.0.get("registration_resources").and_then(serde_json::Value::as_array).is_some_and(|rows|
        rows.iter().any(|row|row.get("role").and_then(serde_json::Value::as_str)==Some(authored::registration::logic_dependency::ROLE))))
}
pub(crate) fn public_run_is_selected(raw:&str)->Result<bool,String>{let input:FirstCallableRun=request(raw)?;artifact_is_selected(&input.source_artifact_utf8)}
pub(super) fn public_open_is_selected(raw:&str)->Result<bool,String>{let input:Open=request(raw)?;artifact_is_selected(&input.artifact_text)}
pub(super) fn public_native_is_selected(raw:&str)->Result<bool,String>{let input:FirstCallableNative=request(raw)?;artifact_is_selected(&input.source_artifact_utf8)}

pub(crate) fn operate_private(operation:&str,raw:&str)->Result<String,String>{
    match operation{
        "source_probe"=>probe(raw),"source_prepare"=>prepare(raw),"source_open"=>open(raw),
        "public_callable_source_run"=>run(raw),"public_callable_source_save_native"=>save_native(raw),
        "public_callable_source_reopen_native"=>reopen_native(raw),
        _=>Err("E_MULTILANG_OPERATION_CONTEXT: 이 공급자는 기존 B1 소비 작업만 받습니다".into()),
    }
}
fn prepare(raw:&str)->Result<String,String>{prepare_selected(raw,None)}
fn prepare_selected(raw:&str,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    let input:Prepare=request(raw)?;bytes(&input.expected_parent_hex,true)?;let revision=bytes(&input.revision_hex,false)?;
    let project=authored::open_with_diagnostic(input.artifact_json_utf8.as_bytes(),diagnostic.as_deref_mut())?;
    let Some((_record,context))=selected_context(&project,diagnostic.as_deref_mut())?else{return super::prepare_diagnostic(raw,diagnostic)};
    if !bytes(&input.expected_parent_hex,true)?.is_empty()||input.previous_source_artifact_utf8.is_some(){return Err("E_SOURCE_SAVE_PARENT: 첫 B1 저장만 지원하며 갱신은 포함하지 않습니다".into());}
    let source=if let Some(output)=diagnostic.as_deref_mut(){
        crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic_diagnostic(input.artifact_json_utf8.as_bytes(),context)
            .map_err(|error|{let message=error.message.clone();if output.is_none(){*output=Some(error);}message})?
    }else{crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_logic(input.artifact_json_utf8.as_bytes(),context)?};
    let mut draft=authored::registration::prepare_precanon_under_candidate_owner(&source,None,&mut |bytes|getrandom::fill(bytes).map_err(|_|"E_AUTHORED_ID_RANDOM".into()))?;
    draft.attach_logic_dependency(&source,&project.source_snapshot.entry_path)?;
    let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft)?;
    Source::prepare_logic_private(&owned,&[],&revision)?.artifact()
}
fn open(raw:&str)->Result<String,String>{
    let input:Open=request(raw)?;let source=Source::from_artifact_logic_private(&input.artifact_text)?;
    if source.project()!=bytes(&input.project_hex,false)?||source.parent()!=bytes(&input.parent_hex,true)?||source.revision()!=bytes(&input.revision_hex,false)?{
        return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 원문 저장본과 실제 저장 머리가 다릅니다".into());
    }
    source.owned().project().save()
}
fn run(raw:&str)->Result<String,String>{run_selected(raw,None)}
fn run_selected(raw:&str,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    let input:FirstCallableRun=request(raw)?;let source=Source::from_artifact_logic_private(&input.source_artifact_utf8)?;
    let owned=source.recorded_execution_owner()?;let precanon=owned.precanon_view();
    let mut program=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(precanon.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target())?;
    authored::registration::bind_recorded_logic_source_program(&precanon,&mut program,owned.registration().ok_or("E_LOGIC_STORAGE_CONTEXT_REQUIRED")?)?;
    let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
    let result=crate::ddn_runtime::DdnRunner::new(program,owned.sources().entry_declaration().name()).run_update_diagnostic(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).map_err(|error|{
        let located_operand = match &error {
            crate::ddn_runtime::EvalError::SourceLocated { error, span, .. }
                if matches!(error.as_ref(), crate::ddn_runtime::EvalError::DivisionByZero
                    | crate::ddn_runtime::EvalError::ArithmeticFault(ddonirang_core::ArithmeticFaultKind::DivByZero)
                    | crate::ddn_runtime::EvalError::ConditionType) =>
                Some((*span, error.diagnostic_parts().1)),
            _ => None,
        };
        let mut failure=crate::runtime_surface::execution_error_diagnostic(error,&owned.project().source_snapshot.entry_path);
        if let Some(((start, end), message)) = located_operand {
            failure.message = message;
            failure.span = None;
            if let Ok((path, span)) = precanon.sources().preparation()
                .original_span(ddonirang_lang::Span::new(start, end))
            {
                if span.start < span.end {
                    failure = failure.remap_source(path, (span.start, span.end));
                }
            }
        }
        let message=failure.message.clone();if let Some(output)=diagnostic.as_deref_mut(){if output.is_none(){*output=Some(failure);}}message
    })?;
    // Reuse common result fields; never forge a user-original v4 binding.
    serde_json::to_string(&serde_json::json!({"output_log_texts":result.display_outputs.iter().map(|o|o.text.clone()).collect::<Vec<_>>(),"source_start_identity_hex":hex::encode(owned.definition_identity().as_bytes()),"source_only":true})).map_err(|_|"E_MULTILANG_RESULT_ENCODING".into())
}
#[cfg(not(target_arch="wasm32"))]
fn native_input(raw:&str)->Result<(FirstCallableNative,Source),String>{
    let input:FirstCallableNative=request(raw)?;let source=Source::from_artifact_logic_private(&input.source_artifact_utf8)?;Ok((input,source))
}
#[cfg(not(target_arch="wasm32"))]
fn save_native(raw:&str)->Result<String,String>{
    use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,source_revision::SourceSaveFailure};
    let (input,source)=native_input(raw)?;
    let mut adapter=SqliteAdapterV1::provision_source_revision(&input.storage_root).map_err(|e|format!("E_SOURCE_SAVE_BEFORE_COMMIT: {e:?}"))?;
    let read=adapter.save_initial_logic_source_revision_private(&source).map_err(|e|match e{
        SourceSaveFailure::BeforeCommit(cause)=>format!("E_SOURCE_SAVE_BEFORE_COMMIT: {cause:?}"),
        SourceSaveFailure::AfterCommitUnconfirmed(cause)=>format!("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 뒤 확인되지 않았습니다 ({cause:?})"),
    })?;
    let actual=read.source().artifact()?;if actual!=input.source_artifact_utf8{return Err("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 후 원문이 다릅니다".into());}Ok(actual)
}
#[cfg(not(target_arch="wasm32"))]
fn reopen_native(raw:&str)->Result<String,String>{
    let (input,expected)=native_input(raw)?;
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&input.storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let read=adapter.load_current_logic_source_revision_private(expected.project(),expected.revision(),expected.integrity()).map_err(|e|format!("E_SOURCE_REVISION_READBACK: {e:?}"))?;
    let actual=read.source().artifact()?;if actual!=input.source_artifact_utf8{return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 저장된 전체 원문·정체와 기대 문맥이 다릅니다".into());}Ok(actual)
}
#[cfg(target_arch="wasm32")]
fn save_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED".into())}
#[cfg(target_arch="wasm32")]
fn reopen_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED".into())}

/// Existing DTOs only. Actual fresh parser and explicit-run diagnostics.
/// Native/readback custody errors do not claim source positions or admission.
pub(crate) fn operate_private_diagnostic(operation:&str,raw:&str)->Result<String,crate::preprocess::ExecutionDiagnostic>{
    let mut diagnostic=None;
    let result=match operation{
        "source_probe"=>probe_selected(raw,Some(&mut diagnostic)),
        "source_prepare"=>prepare_selected(raw,Some(&mut diagnostic)),
        "public_callable_source_run"=>run_selected(raw,Some(&mut diagnostic)),
        _=>return Err(crate::preprocess::ExecutionDiagnostic{
            code:"E_MULTILANG_OPERATION_CONTEXT",message:"이 진단 연결은 원문 조회·준비·명시 실행만 받습니다".into(),
            source_path:String::new(),span:None,phase:"source_prepare",cause:None,offending_field:None,expected:None,actual:None}),
    };
    result.map_err(|message|diagnostic.unwrap_or_else(||authored::registration::logic_dependency::dependency_diagnostic(message,
        if operation=="public_callable_source_run"{"source_restore"}else{"source_prepare"})))
}
