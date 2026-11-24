//! Workbench source-only candidate transport. Actual strict IndexedDB CAS and
//! full recorded readback follow this authoring step; it cannot start a world.
pub(crate) mod logic_consumer;
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Prepare {artifact_json_utf8:String,expected_parent_hex:String,revision_hex:String,previous_source_artifact_utf8:Option<String>}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Open {artifact_text:String,project_hex:String,parent_hex:String,revision_hex:String}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {artifact_json_utf8:String}
// Select the existing model frontdoor only under its validated profile.
// Classification neither runs the model nor grants a Native root, durable
// evidence, registration or a world continuation.
fn checked_standalone_model(profile:&str,path:&str,source:&str)->Result<bool,String> {
    if profile!=crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY
        || !ddonirang_lang::has_model_surface(source) {return Ok(false);}
    let program=ddonirang_lang::parse_model_program_with_source_file(source,path)
        .map_err(|error|error.to_string())?;
    if let Some(error)=program.ir.diagnostics.iter()
        .find(|item|matches!(item.severity,"error"|"error_candidate")) {
        return Err(format!("{} [{}..{}]: {}",error.candidate_code,
            error.provenance.start,error.provenance.end,error.message));
    }
    Ok(true)
}
fn nominal(project:&super::ProjectArtifact)->Result<bool,String>{nominal_diagnostic(project,None)}
fn nominal_diagnostic(project:&super::ProjectArtifact,diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<bool,String> {
    inspect_source(project,diagnostic,false)
}
fn inspect_source(project:&super::ProjectArtifact,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    check_ordinary_j1:bool,
)->Result<bool,String> {
    // Syntax is selected by the validated artifact, never inferred from text.
    let j1=if project.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE {
        Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&project.source_profile)
            .map_err(|error|error.to_string())?)
    }else{None};
    // Resolve imports with the existing whole-work owner; do not execute Start.
    if let Some(prepared)=project.authored_preparation_diagnostic(diagnostic.as_deref_mut())? {
        let parsed=match j1 {
            Some(profile)=>ddonirang_lang::Parser::parse_j1_v25_with_mode(
                &prepared.source,&prepared.target_path,profile,ddonirang_lang::ParseMode::Strict),
            None=>ddonirang_lang::parse_v25_with_mode(&prepared.source,&prepared.target_path,
                ddonirang_lang::ParseMode::Strict),
        }.map_err(|error|{
            if let Some(out)=diagnostic.as_deref_mut(){if out.is_none(){*out=prepared.mapped_parse_diagnostic(&error);}}
            error.to_string()
        })?;
        let requires_registration=parsed.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|
            matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)))
            || super::registration::requires_resolved_program(&parsed);
        if let Some(profile)=j1.filter(|_|check_ordinary_j1&&!requires_registration){
            crate::ddn_runtime::DdnProgram::from_linked_j1_preparation(
                &prepared,profile,ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),diagnostic,
            )?;
        }
        return Ok(requires_registration);
    }
    // Rootless sources keep their existing standalone parsing contract.
    let mut found=false;
    for module in &project.authored_identity.modules {for path in &module.sources {
        let source=project.source_snapshot.files.iter().find(|file|file.path==*path).ok_or("E_SOURCE_SAVE_MODULE_SOURCE")?;
        if checked_standalone_model(&project.source_profile,path,&source.text)? {continue;}
        let parsed=match j1 {
            Some(profile)=>ddonirang_lang::Parser::parse_j1_v25_with_mode(
                &source.text,path,profile,ddonirang_lang::ParseMode::Strict),
            None=>ddonirang_lang::parse_v25_with_mode(&source.text,path,ddonirang_lang::ParseMode::Strict),
        }.map_err(|error|error.to_string())?;
        found|=parsed.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)));
        found|=super::registration::requires_resolved_program(&parsed);
    }}
    Ok(found)
}
pub(super) fn requirements(raw:&str)->Result<String,String>{requirements_diagnostic(raw,None)}
pub(super) fn requirements_diagnostic(raw:&str,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Probe=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let project=super::open_with_diagnostic(request.artifact_json_utf8.as_bytes(),diagnostic.as_deref_mut())?;
    Ok(nominal_diagnostic(&project,diagnostic)?.to_string())
}
fn bytes(text:&str,empty:bool)->Result<Vec<u8>,String>{
    if (!empty&&text.is_empty())||text.len()>8192||text.len()%2!=0||!text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
        return Err("E_SOURCE_SAVE_IDENTITY".into());
    }
    hex::decode(text).map_err(|_|"E_SOURCE_SAVE_IDENTITY".into())
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirstCallablePrepare {
    original_project_json_utf8:String,original_source_artifact_utf8:String,original_source_path:String,
    ddl_source:super::FirstCallableDdlSource,consumer_project_json_utf8:String,consumer_source_path:String,revision_hex:String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirstCallableOriginalPrepare {
    original_project_json_utf8: String,
    original_source_path: String,
    ddl_source: super::FirstCallableDdlSource,
    revision_hex: String,
}

/// Register a fresh approved original through the existing owner and issuer.
/// This is initial source preparation, not DDL activation or Native storage.
pub(super) fn prepare_first_callable_original(raw: &str) -> Result<String, String> {
    if raw.len() > 16_000_000 { return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into()); }
    let strict: super::StrictValue = serde_json::from_str(raw)
        .map_err(|_| "E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request: FirstCallableOriginalPrepare = serde_json::from_value(strict.0)
        .map_err(|_| "E_SOURCE_SAVE_REQUEST_FIELDS")?;
    // Keep the existing export's bounded original material; validate revision
    // before entering the issuer. No caller-supplied identity or context.
    if request.original_project_json_utf8.len() > 1_048_576 {
        return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());
    }
    bytes(&request.revision_hex, false)?;
    let original = super::open(request.original_project_json_utf8.as_bytes())?;
    let owner = source_module(&original, &request.original_source_path)?;
    let registry = ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
    let first = original.first_callable_registration_context(
        owner.id.as_str(), &request.original_source_path,
        &request.ddl_source.text, &request.ddl_source.path,
        registry.language_profile_identity, registry.registration_revision,
    )?;
    let original_text = original.save()?;
    let prepared = prepare_first_callable(&serde_json::json!({
        "artifact_json_utf8": original_text, "expected_parent_hex": "",
        "revision_hex": request.revision_hex, "previous_source_artifact_utf8": null,
    }).to_string(), &first)?;
    // The existing export audits the real issued Definition/Pin/body facts.
    // Do not publish a source receipt for a callable outside this closed scope.
    first.produce_public_ddl(&original_text, &prepared)?;
    Ok(prepared)
}

fn source_module<'a>(project:&'a super::ProjectArtifact,path:&str)->Result<&'a super::AuthoredModule,String>{
    let mut owners=project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path));
    let owner=owners.next().ok_or("E_MULTILANG_SOURCE_OWNER_REQUIRED")?;
    if owners.next().is_some(){return Err("E_MULTILANG_SOURCE_OWNER_AMBIGUOUS".into());}Ok(owner)
}
pub(super) fn prepare_first_callable_product(raw:&str)->Result<String,String>{
    prepare_first_callable_product_with_analysis(raw).map_err(|e|e.message)
}
pub(super) fn prepare_first_callable_product_with_analysis(raw:&str)->Result<String,super::FirstCallableProductError>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:FirstCallablePrepare=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let original=super::open(request.original_project_json_utf8.as_bytes())?;
    let owner=source_module(&original,&request.original_source_path)?;
    let registry=ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
    let first=original.first_callable_registration_context(owner.id.as_str(),&request.original_source_path,&request.ddl_source.text,&request.ddl_source.path,registry.language_profile_identity,registry.registration_revision)?;
    let export=first.produce_public_ddl(&request.original_project_json_utf8,&request.original_source_artifact_utf8)?;
    let consumer=super::open(request.consumer_project_json_utf8.as_bytes())?;
    let owner=source_module(&consumer,&request.consumer_source_path)?;
    let activated=export.activate_for_consumer(&consumer,owner.id.as_str(),&request.consumer_source_path)?;
    let attempted:Result<String,String>=(||{
        let v3=export.attach_v3(&consumer,owner.id.as_str(),&request.consumer_source_path)?;
        let first=v3.restore_language_context(export.original_registration_record()?)?;
        prepare_first_callable(&serde_json::json!({"artifact_json_utf8":v3.save()?,"expected_parent_hex":"","revision_hex":request.revision_hex,"previous_source_artifact_utf8":null}).to_string(),&first)
    })();
    attempted.map_err(|message|{
        // Retain this exact attempt's checked owner/context. Never run prepare
        // or issue another registry/capture identity to obtain display data.
        let file=consumer.source_snapshot.files.iter().find(|f|f.path==request.consumer_source_path).unwrap();
        let generation=0;
        let checked=crate::source_analysis::analyze_source_with_first_callable_context(crate::source_analysis::SourceAnalysisRequest{
            source_text:file.text.clone(),path:file.path.clone(),analysis_generation:generation,source_profile_identity:consumer.source_profile.clone(),
        },activated.compiler_context().clone());
        let current=checked.is_current_for(&file.text,generation,activated.compiler_context());
        super::FirstCallableProductError{message,analysis:(current&&checked.snapshot().outcome=="REJECTED").then(||checked.snapshot().clone())}
    })
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirstCallableRun{source_artifact_utf8:String}
/// Explicit, isolated source evaluation, not a Native RootGrant/world session.
/// No existing editor/world or storage locator is accepted by this operation.
pub(super) fn run_first_callable_product(raw:&str)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:FirstCallableRun=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=Source::from_artifact(&request.source_artifact_utf8)?;
    let owned=source.recorded_execution_owner()?;
    let first=owned.sources().first_callable_context().ok_or("E_DDL_V3_RECORDED_CONTEXT_REQUIRED")?;
    let path=&owned.project().source_snapshot.entry_path;
    let file=owned.project().source_snapshot.files.iter().find(|f|&f.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
    let program=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&file.text,path,first.clone())?;
    let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
    let result=crate::ddn_runtime::DdnRunner::new(program,owned.sources().entry_declaration().name()).run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).map_err(|e|e.to_string())?;
    serde_json::to_string(&serde_json::json!({"output_log_texts":result.display_outputs.iter().map(|o|o.text.clone()).collect::<Vec<_>>(),
        "source_start_identity_hex":hex::encode(owned.definition_identity().as_bytes()),"original_binding":first.saved_dependency_material()?,"source_only":true})).map_err(|_|"E_MULTILANG_RESULT_ENCODING".into())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirstCallableNative{source_artifact_utf8:String,storage_root:std::path::PathBuf}
#[cfg(not(target_arch="wasm32"))]
fn native_request(raw:&str)->Result<(FirstCallableNative,Source),String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:FirstCallableNative=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=Source::from_artifact(&request.source_artifact_utf8)?;
    if source.owned().sources().first_callable_context().is_none(){return Err("E_DDL_V3_RECORDED_CONTEXT_REQUIRED".into());}
    Ok((request,source))
}
#[cfg(not(target_arch="wasm32"))]
pub(super) fn save_first_callable_native(raw:&str)->Result<String,String>{
    use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,source_revision::SourceSaveFailure};
    let (request,source)=native_request(raw)?;
    if !source.parent().is_empty(){return Err("E_SOURCE_SAVE_PARENT: 첫 원문 저장에는 빈 부모만 허용합니다".into());}
    let mut adapter=SqliteAdapterV1::provision_source_revision(&request.storage_root).map_err(|e|format!("E_SOURCE_SAVE_BEFORE_COMMIT: {e:?}"))?;
    let read=adapter.save_initial_source_revision(&source).map_err(|e|match e{
        SourceSaveFailure::BeforeCommit(cause)=>format!("E_SOURCE_SAVE_BEFORE_COMMIT: {cause:?}"),
        SourceSaveFailure::AfterCommitUnconfirmed(cause)=>format!("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 뒤 확인되지 않았습니다 ({cause:?})"),
    })?;
    let actual=read.source().artifact()?;
    if actual!=request.source_artifact_utf8{return Err("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 후 원문이 다릅니다".into());}
    Ok(actual)
}
#[cfg(not(target_arch="wasm32"))]
pub(super) fn reopen_first_callable_native(raw:&str)->Result<String,String>{
    let (request,expected)=native_request(raw)?;
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&request.storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let read=adapter.load_current_source_revision(expected.project(),expected.revision(),expected.integrity()).map_err(|e|format!("E_SOURCE_REVISION_READBACK: {e:?}"))?;
    let actual=read.source().artifact()?;
    if actual!=request.source_artifact_utf8{return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 저장된 전체 원문·정체와 기대 문맥이 다릅니다".into());}
    Ok(actual)
}
#[cfg(target_arch="wasm32")]
pub(super) fn save_first_callable_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED: Native 원문 저장은 이 실행면에서 지원하지 않습니다".into())}
#[cfg(target_arch="wasm32")]
pub(super) fn reopen_first_callable_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED: Native 원문 재열기는 이 실행면에서 지원하지 않습니다".into())}

pub(super) fn prepare(raw:&str)->Result<String,String>{
    if logic_consumer::prepare_is_selected(raw)?{return logic_consumer::operate_private("source_prepare",raw);}
    prepare_selected(raw,None,None)
}
pub(super) fn prepare_first_callable(raw:&str,first:&super::FirstCallableAuthoredContext)->Result<String,String>{
    prepare_selected(raw,Some(first),None)
}
fn restore_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>)->Result<Source,String>{
    match first {Some(first)=>Source::from_artifact_with_first_callable(raw,first),None=>Source::from_artifact(raw)}
}
pub(super) fn prepare_diagnostic(raw:&str,diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    prepare_selected(raw,None,diagnostic)
}
fn prepare_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>,mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Prepare=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let parent=bytes(&request.expected_parent_hex,true)?;let revision=bytes(&request.revision_hex,false)?;
    let project=super::open_with_diagnostic(request.artifact_json_utf8.as_bytes(),diagnostic.as_deref_mut())?;
    let v3=project.schema==super::SCHEMA_V3;
    if v3 {first.ok_or("E_DDL_V3_RECORDED_CONTEXT_REQUIRED")?.registration_record_for_consumer(&project)?;}
    // Preserve established authoring/runtime workflows while VC04's source
    // owner is wired. This new UI route is nominal roots + their retained
    // catalogs only; it is not a claim that every source-save route is closed.
    if first.is_none(){
        let check_ordinary_j1=parent.is_empty()&&request.previous_source_artifact_utf8.is_none();
        if !inspect_source(&project,diagnostic.as_deref_mut(),check_ordinary_j1)?
            &&request.previous_source_artifact_utf8.is_none(){
            return Ok("null".into());
        }
    }
    if project.source_snapshot.root_path.is_none(){return Err("E_VC04_SOURCE_ROOT_REQUIRED: 임자·이름씨 저장에는 명시 얽음 뿌리와 시작 정의가 필요합니다".into());}
    if !v3&&first.is_some_and(|first|first.original_artifact()!=&project){return Err("E_MULTILANG_ORIGINAL_OWNER: 선택한 원본과 저장 작품이 다릅니다".into());}
    let previous=request.previous_source_artifact_utf8.as_deref().map(|raw|restore_selected(raw,first)).transpose()?;
    let project_id=hex::decode(project.authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_SOURCE_SAVE_PROJECT_IDENTITY")?;
    if previous.as_ref().is_some_and(|p|p.project()!=project_id||p.revision()!=parent) {
        return Err("E_SOURCE_SAVE_PARENT: 다른 작품이거나 저장 부모가 다릅니다".into());
    }
    if previous.is_none()&&!parent.is_empty(){return Err("E_SOURCE_SAVE_PARENT_REQUIRED".into());}
    if let Some(old)=&previous {if old.owned().project().save()?==request.artifact_json_utf8{return old.artifact();}}
    let source=if v3 {
        crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source_with_first_callable(request.artifact_json_utf8.as_bytes(),std::sync::Arc::new(first.unwrap().clone()))?
    }else{crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(request.artifact_json_utf8.as_bytes())?};
    if v3 {
        let owned=crate::project_revision_envelope_supply::finalize_first_callable_consumer_source(source)?;
        return Source::prepare(&owned,&parent,&revision)?.artifact();
    }
    let mut entropy=|id:&mut [u8;32]|getrandom::fill(id).map_err(|_|"E_SOURCE_SAVE_ENTROPY".into());
    let old=previous.as_ref().and_then(|p|p.owned().registration());
    let draft=match first {
        Some(first)=>super::registration::prepare_public_callables_under_candidate_owner(&source,first.module_identity().as_str(),old,&mut entropy)?,
        None=>super::registration::prepare_precanon_under_candidate_owner(&source,old,&mut entropy)?,
    };
    let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft)?;
    Source::prepare(&owned,&parent,&revision)?.artifact()
}
pub(super) fn open(raw:&str)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Open=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let selector:serde_json::Value=serde_json::from_str(&request.artifact_text).map_err(|_|"E_SOURCE_ARTIFACT_FIELDS")?;
    if selector.get("schema").and_then(serde_json::Value::as_str)==Some(crate::unit_wire::project_runtime_attachment::code_source::SCHEMA){
        let code=crate::unit_wire::project_runtime_attachment::code_source::CodeSource::decode(&request.artifact_text)?;
        if code.project_hex!=request.project_hex||code.parent_hex!=request.parent_hex||code.revision_hex!=request.revision_hex{return Err("E_CODE_SOURCE_READBACK_CONTEXT".into());}
        return Ok(code.authored_project_json_utf8);
    }
    if selector.get("schema").and_then(serde_json::Value::as_str)==Some(crate::unit_wire::project_runtime_attachment::nominal_source::SCHEMA){
        let source=crate::unit_wire::project_runtime_attachment::nominal_source::NominalSource::decode(&request.artifact_text)?;
        if source.project_hex!=request.project_hex||source.parent_hex!=request.parent_hex||source.revision_hex!=request.revision_hex{return Err("E_NOMINAL_SOURCE_SELECTED_REVISION".into());}
        return Ok(source.authored_project_json_utf8);
    }
    if selector.get("schema").and_then(serde_json::Value::as_str)==Some(crate::unit_wire::project_runtime_attachment::model_source::SCHEMA){
        let model=crate::unit_wire::project_runtime_attachment::model_source::ModelSource::decode(&request.artifact_text)?;
        if model.project_hex!=request.project_hex||model.parent_hex!=request.parent_hex||model.revision_hex!=request.revision_hex{return Err("E_MODEL_SOURCE_READBACK_CONTEXT".into());}
        return Ok(model.authored_project_json_utf8);
    }

    if logic_consumer::public_open_is_selected(raw)?{return logic_consumer::operate_private("source_open",raw);}
    open_selected(raw,None)
}
pub(super) fn open_first_callable(raw:&str,first:&super::FirstCallableAuthoredContext)->Result<String,String>{open_selected(raw,Some(first))}
fn open_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Open=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=restore_selected(&request.artifact_text,first)?;
    if source.project()!=bytes(&request.project_hex,false)?||source.parent()!=bytes(&request.parent_hex,true)?||source.revision()!=bytes(&request.revision_hex,false)? {
        return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 원문 저장본과 실제 저장 머리가 다릅니다".into());
    }
    match source.owned().sources().public_callable_artifact(){
        Some(artifact)=>artifact.save(),None=>source.owned().project().save(),
    }
}

/// Candidate construction for the distinct Native authoring transition. The
/// actual protected caller must supply/admit its world or source parent; this
/// does not relax `source_prepare`'s existing source-only parent contract.
pub(crate) fn prepare_authoring_source(previous:&Source,artifact:&str,parent:&[u8],revision:&[u8])->Result<Source,String>{
    let project=super::open(artifact.as_bytes())?;
    if hex::decode(project.authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_SOURCE_SAVE_PROJECT_IDENTITY")?!=previous.project()
        ||project.source_profile.as_bytes()!=previous.profile()||project.source_snapshot.root_path.is_none(){return Err("E_SOURCE_SAVE_PARENT: 다른 작품·글판 또는 명시 뿌리 없는 원문입니다".into());}
    if previous.owned().project().save()?==artifact{return Err("E_SOURCE_AUTHORING_UNCHANGED: 같은 원문은 검증된 재저장 경로를 쓰세요".into());}
    let source=crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(artifact.as_bytes())?;
    let mut entropy=|id:&mut [u8;32]|getrandom::fill(id).map_err(|_|"E_SOURCE_SAVE_ENTROPY".into());
    let draft=super::registration::prepare_precanon_under_candidate_owner(&source,previous.owned().registration(),&mut entropy)?;
    let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft)?;
    Source::prepare(&owned,parent,revision)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project(body:&str)->String {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:body.into()}]},String::new()).unwrap().save().unwrap()
    }

    fn j1_project(rooted:bool)->String {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let body="#!日本語\n実行:手続き := { 真 表示. }.";
        let mut files=vec![SourceFile{path:"main.ddn".into(),text:body.into()}];
        if rooted {files.push(SourceFile{path:"root.ddn".into(),text:
            "#!日本語\n(版=\"v25\", モジュール群=列[(名前=アプリ, 見出し=\"main.ddn\", 原文=\"main.ddn\")], 開始=(モジュール=アプリ, 定義=実行)) 構成.".into()});}
        ProjectArtifact::create(ddonirang_lang::ddl::J1_SOURCE_PROFILE.into(),SourceSnapshot{
            root_path:rooted.then(||"root.ddn".into()),entry_path:"main.ddn".into(),files,
        },if rooted{String::new()}else{"アプリ".into()}).unwrap().save().unwrap()
    }
    #[test]
    fn r31_j1_source_preflight_preserves_explicit_profile_and_existing_route() {
        for rooted in [false,true] {
            let raw=j1_project(rooted);
            assert_eq!(requirements(&serde_json::json!({"artifact_json_utf8":raw}).to_string()).unwrap(),"false");
            // No registration need: retain the established null selection.
            assert_eq!(prepare(&request(&raw,"","01",None)).unwrap(),"null");
        }
    }
    #[test]
    fn r31_j1_source_preflight_rejects_bad_header_and_fullwidth_without_fallback() {
        for (from,to) in [("#!日本語","#!한국어"),("真 表示.","真 表示。")]{
            let mut project=super::super::open(j1_project(false).as_bytes()).unwrap();
            project.source_snapshot.files[0].text=project.source_snapshot.files[0].text.replace(from,to);
            let error=project.save().and_then(|raw|requirements(&serde_json::json!({"artifact_json_utf8":raw}).to_string())).unwrap_err();
            if to.contains('。'){assert!(error.contains("E_MULTILANG_FULLWIDTH_SYNTAX"),"{error}");}
        }
    }
    fn ac_imported_work()->String { r####"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"18ee9053cd02b4b5de2a27cf91b9e59a48d8b87e5ef5c9ca8fdb31741f2f643c","modules":[{"compositions":[],"head":"main.ddn","id":"0343b2c1d7206df36bb83b0a2c89a182f997196f9535e591b5a25e7e606b9e8f","name":"앱","sources":["main.ddn"]},{"compositions":[],"head":"정리.ddn","id":"b8b07a724015ab4fab857b748b5c45cddc57298497869b4c3a9c0f6effd2e14b","name":"정리","sources":["정리.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"938f19fdf7d5f2b59438ecbf9124f8fa6e027aeb98e7ad7a3ebf96df1bf44719","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"main.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\"),(이름=정리, 머리=\"정리.ddn\", 원문=\"정리.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음."},{"path":"main.ddn","text":"(수학=정리) 쓰임.\n(미지수:나눔수~을, 근거:(2 * (미지수 + 3) =:= 14)보임~로)\n첫소비보이다{첫소비보이기}:(미지수 =:= 4)보임:셈씨 := {\n    첫소비보이다 <- (미지수=미지수, 근거=근거) 수학.괄호결론보이기.\n}.\n\n실행:움직씨 := { }.\n"},{"path":"정리.ddn","text":"(괄호결론보이기=괄호결론보이다) 드러냄.\n(미지수:나눔수~을, 근거:(2 * (미지수 + 3) =:= 14)보임~로)\n괄호결론보이다{괄호결론보이기}:(미지수 =:= 4)보임:셈씨 := {\n    펼침 := (근거=근거, 왼=2 * 미지수 + 6, 오른=14) 등식정리.\n    빼기 := (근거=펼침, 항=6) 양변빼기.\n    정리 := (근거=빼기, 왼=2 * 미지수, 오른=8) 등식정리.\n    나누기 := (근거=정리, 계수=2) 양변나누기.\n    괄호결론보이다 <- (근거=나누기, 왼=미지수, 오른=4) 등식정리.\n}.\n\n"}],"root_path":"root.ddn"}}"####.into() }
    #[test]
    fn ac_source_probe_whole_work_and_prepare() {
        let raw=ac_imported_work();
        assert_eq!(requirements(&serde_json::json!({"artifact_json_utf8":raw}).to_string()).unwrap(),"false");
        let saved=prepare(&request(&raw,"","01",None)).unwrap();
        // No nominal/sum declaration: keep the established browser route.
        // Native source saving is selected separately by the actual host.
        assert_eq!(saved,"null");
    }
    #[test]
    fn ac_source_probe_bad_imports() {
        for (from,to) in [("수학.괄호결론보이기","수학.없는보이기"),("(괄호결론보이기=괄호결론보이다) 드러냄.","")] {
            let mut p=super::super::open(ac_imported_work().as_bytes()).unwrap();
            for f in &mut p.source_snapshot.files {f.text=f.text.replace(from,to);}
            let raw=p.save().unwrap();
            assert!(requirements(&serde_json::json!({"artifact_json_utf8":raw}).to_string()).is_err());
            assert!(prepare(&request(&raw,"","01",None)).is_err());
        }
    }
    #[test]
    fn ac_source_probe_no_start_and_rootless() {
        let raw=project("값:바른수:셈씨 := { 값 <- 3. }. 실행:움직씨 := { }.");
        let mut p=super::super::open(raw.as_bytes()).unwrap();
        p.source_snapshot.files[0].text=p.source_snapshot.files[0].text.replace(", 시작=(모듈=앱, 정의=실행)","");
        assert!(requirements(&serde_json::json!({"artifact_json_utf8":p.save().unwrap()}).to_string()).is_ok());
        let raw=super::super::ProjectArtifact::create("v1-core-v25".into(),super::super::SourceSnapshot {
            root_path:None,entry_path:"main.ddn".into(),files:vec![super::super::SourceFile{path:"main.ddn".into(),text:"값:바른수:셈씨 := { 값 <- 3. }.".into()}]},"앱".into()).unwrap().save().unwrap();
        assert!(requirements(&serde_json::json!({"artifact_json_utf8":raw}).to_string()).is_ok());
    }
    fn request(artifact:&str,parent:&str,revision:&str,previous:Option<&str>)->String {
        serde_json::json!({"artifact_json_utf8":artifact,"expected_parent_hex":parent,"revision_hex":revision,"previous_source_artifact_utf8":previous}).to_string()
    }
    fn ids(source:&Source)->Vec<Vec<u8>> {
        let draft=source.owned().registration().unwrap();
        let parsed=crate::registered_atom_contract::source_binding::Candidate::parse(draft.descriptor()).unwrap();
        let mut out=Vec::new();
        for schema in parsed.nominal.unwrap().schemas {
            out.extend([schema.module.to_vec(),schema.definition.to_vec(),schema.name_type.to_vec()]);
            out.extend(schema.slots.into_iter().map(|slot|slot.identity.to_vec()));
        }
        out
    }
    #[test]
    fn r3_1_source_catalog_actual_prepare_recorded_reopen_preserves_ids_and_pesi() {
        let artifact=project("선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 실행:움직씨 := { 자료 <- 선택.$쌍(세로=4,가로=3). 자료에 따라 { 선택.$쌍(가로,세로)이면 { (가로+세로) 보여주기. }. }. }.");
        assert_eq!(requirements(&serde_json::json!({"artifact_json_utf8":artifact}).to_string()).unwrap(),"true");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();
        let first=Source::from_artifact(&sealed).unwrap();
        let restored=Source::from_artifact(&sealed).unwrap();
        assert_eq!(first.pesi(),restored.pesi());assert_eq!(first.envelope(),restored.envelope());assert_eq!(first.payload(),restored.payload());
        assert_eq!(first.owned().definition_identity(),restored.owned().definition_identity());
        let records=first.owned().registration().unwrap().recorded_blobs();
        let raw=serde_json::to_vec(&records).unwrap();
        assert_eq!(serde_json::to_vec(&restored.owned().registration().unwrap().recorded_blobs()).unwrap(),raw);
        assert_eq!(prepare(&request(&artifact,"01","02",Some(&sealed))).unwrap(),sealed);
        let blobs=first.owned().registration().unwrap().artifacts();
        let catalog=blobs.iter().find(|blob|blob.role()==Some("resolved-supply-catalog-r3.3")).unwrap();
        let text=std::str::from_utf8(catalog.bytes()).unwrap();
        assert!(text.starts_with("{\"schema\":\"ddn.resolved-supply-catalog.r3.3\""));
        assert!(text.contains("\"kind\":\"callable-context\""));
        let source=first.owned().precanon_view();
        let old=crate::saved_definition_identity::issue_saved_start(source.sources(),source.project(),0).unwrap();
        assert_ne!(first.owned().definition_identity(),&old,"catalog/builtin binding must change the saved Start context, not remain a cache");
        let mut executable=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(first.owned().sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        executable.bind_registered_project_context(first.owned()).unwrap();
        assert_eq!(executable.canonical_ddn(),first.owned().registration().unwrap().resolved_start_material(&source).unwrap().unwrap().0,
            "actual runtime canon must use the same raw pin projection as the source/Start owner");
    }
    #[test]
    fn r3_1_source_catalog_builtin_result_uses_fixed_manifest_without_user_rows() {
        let artifact=project("실행:움직씨 := { (바른수,글)맺음.$잘됨(값=3)에 따라 { (바른수,글)맺음.$잘됨(값)이면 { 값 보여주기. }. (바른수,글)맺음.$안됨(이유)이면 { 이유 보여주기. }. }. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let first=Source::from_artifact(&sealed).unwrap();
        let blob=first.owned().registration().unwrap().artifacts().iter().find(|blob|blob.role()==Some("resolved-supply-catalog-r3.3")).unwrap();
        let transport:serde_json::Value=serde_json::from_slice(blob.bytes()).unwrap();
        assert!(transport["branches"].as_array().unwrap().is_empty());assert!(transport["pins"].as_array().unwrap().is_empty());
        assert_eq!(transport["binders"].as_array().unwrap().len(),2);
        assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed);
    }
    #[test]
    fn r3_1_callable_context_parameters_result_and_default_use_actual_type_owner() {
        let artifact=project("선택:고름씨 := { $값(내용:바른수). }. (수치:바른수=2) 표본:바른수:움직씨 := { 표본 <- 수치. 선택.$값(내용=수치)에 따라 { 선택.$값(내용)이면 { 내용 보여주기. }. }. }. 실행:움직씨 := { (수치=3) 표본 보여주기. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let first=Source::from_artifact(&sealed).unwrap();
        let blob=first.owned().registration().unwrap().artifacts().iter().find(|blob|blob.role()==Some("resolved-supply-catalog-r3.3")).unwrap();
        let catalog:serde_json::Value=serde_json::from_slice(blob.bytes()).unwrap();
        let definition=catalog["definitions"].as_array().unwrap().iter().find(|row|row["spelling"]=="표본").unwrap();
        assert_eq!(definition["kind"],"callable-context");assert!(!definition["result_type"].as_str().unwrap().is_empty());
        let pin=catalog["pins"].as_array().unwrap().iter().find(|row|row["owner"]==definition["id"]&&row["spelling"]=="수치").unwrap();
        assert_eq!(pin["required"],false);assert_eq!(pin["default_canon"],"2");assert!(!pin["type"].as_str().unwrap().is_empty());
        assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed);
    }
    #[test]
    fn r3_2_semssi_patterns_record_ordinary_context_without_granting_provider_effect() {
        let artifact=project("선택:고름씨 := { $값(내용:바른수). }. (수치:바른수) 표본:바른수:셈씨 := { 선택.$값(내용=수치)에 따라 { 선택.$값(내용)이면 { 표본 <- 내용. }. }. }. 실행:움직씨 := { (수치=3) 표본 보여주기. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let source=Source::from_artifact(&sealed).unwrap();
        let blob=source.owned().registration().unwrap().artifacts().iter().find(|blob|blob.role()==Some("resolved-supply-catalog-r3.3")).unwrap();
        let catalog:serde_json::Value=serde_json::from_slice(blob.bytes()).unwrap();
        let definition=catalog["definitions"].as_array().unwrap().iter().find(|row|row["spelling"]=="표본").unwrap();
        assert_eq!(definition["kind"],"callable-context");
        assert!(definition["canonical"].as_str().unwrap().contains("셈씨"));
        assert!(catalog["providers"].as_array().unwrap().is_empty());
        assert!(catalog["scopes"].as_array().unwrap().iter().any(|row|row["definition"]==definition["id"]));
        assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed);
        let invalid=project("병사:임자 := { 힘:바른수. }. 선택:고름씨 := { $값(내용:바른수). }. 표본:셈씨 := { 선택.$값(내용=3)에 따라 { 선택.$값(내용)이면 { 병사(힘=내용). }. }. }. 실행:움직씨 := { 표본. }.");
        assert!(prepare(&request(&invalid,"","01",None)).is_err(),"ordinary context registration must not authorize Subject creation in Semssi");
    }
    #[test]
    fn r3_2_existing_structural_product_records_preserve_structural_identity_and_cold() {
        fn proof(body:&str)->Vec<u8> {
            let artifact=project(body);let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let first=Source::from_artifact(&sealed).unwrap();
            assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed);
            let blob=first.owned().registration().unwrap().artifacts().iter().find(|blob|blob.role()==Some("resolved-supply-catalog-r3.3")).unwrap();
            let metadata:serde_json::Value=serde_json::from_slice(blob.bytes()).unwrap();
            let pin=metadata["pins"].as_array().unwrap().iter().find(|row|row["spelling"]=="내용").unwrap();
            hex::decode(pin["type"].as_str().unwrap()).unwrap()
        }
        let first=proof("(가로:셈수,세로:셈수) 점자료:묶음씨 := {}. 선택:고름씨 := { $값(내용:점자료). }. 실행:움직씨 := {}.");
        let reversed=proof("(세로:셈수,가로:셈수) 다른자료:묶음씨 := {}. 선택:고름씨 := { $값(내용:다른자료). }. 실행:움직씨 := {}.");
        let changed=proof("(가로:나눔수,세로:셈수) 점자료:묶음씨 := {}. 선택:고름씨 := { $값(내용:점자료). }. 실행:움직씨 := {}.");
        assert_eq!(first,reversed);assert_ne!(first,changed);assert_eq!(first[b"DDN.R3.TYPEPROOF.1\0".len()],6);
        let dag=proof("(값:셈수) 작은자료:묶음씨 := {}. (왼쪽:작은자료,오른쪽:작은자료) 큰자료:묶음씨 := {}. 선택:고름씨 := { $값(내용:큰자료). }. 실행:움직씨 := {}.");
        assert_eq!(dag[b"DDN.R3.TYPEPROOF.1\0".len()],6,"shared finite children must not be mistaken for an expansion cycle");
    }
    #[test]
    fn r3_1_source_catalog_resealed_kind_mutation_and_missing_catalog_are_refused() {
        use sha2::{Digest,Sha256};
        let artifact=project("선택:고름씨 := { $값(내용:바른수). }. 실행:움직씨 := { 자료 <- 선택.$값(내용=3). 자료에 따라 { 선택.$값(내용)이면 { 내용 보여주기. }. }. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let first=Source::from_artifact(&sealed).unwrap();
        let source=first.owned().precanon_view();
        let original=first.owned().registration().unwrap().recorded_blobs();
        let mut records=serde_json::to_value(&original).unwrap();
        let rows=records.as_array_mut().unwrap();
        let row=rows.iter_mut().find(|row|row["role"]=="resolved-supply-catalog-r3.3").unwrap();
        let bytes=hex::decode(row["raw_hex"].as_str().unwrap()).unwrap();
        let text=String::from_utf8(bytes).unwrap();
        let changed=text.replacen("\"kind\":\"callable-context\"","\"kind\":\"root-context\"",1);
        assert_ne!(changed,text);row["raw_hex"]=hex::encode(changed.as_bytes()).into();
        row["byte_length"]=(changed.len() as u64).into();row["sha256"]=hex::encode(Sha256::digest(changed.as_bytes())).into();
        let records=serde_json::from_value(records).unwrap();
        assert!(super::super::registration::restore_precanon_recorded_candidate(&source,records).is_err());
        let mut missing=serde_json::to_value(&original).unwrap();missing.as_array_mut().unwrap().retain(|row|row["role"]!="resolved-supply-catalog-r3.3");
        let error=super::super::registration::restore_precanon_recorded_candidate(&source,serde_json::from_value(missing).unwrap()).unwrap_err();
        assert!(error.contains("E_RESOLVED_SUPPLY_CATALOG_REQUIRED"),"{error}");
        assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed,"failed readback must not repair the original");
    }
    #[test]
    fn r3_2_resealed_old_schema_and_record_type_trailer_never_restore_or_repair() {
        use sha2::{Digest,Sha256};
        let artifact=project("(가로:셈수,세로:셈수) 점자료:묶음씨 := {}. 선택:고름씨 := { $값(내용:점자료). }. 실행:움직씨 := {}.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let saved=Source::from_artifact(&sealed).unwrap();
        let source=saved.owned().precanon_view();
        let original=saved.owned().registration().unwrap().recorded_blobs();
        for mutation in ["old-schema","type-trailer"] {
            let mut rows=serde_json::to_value(&original).unwrap();
            let row=rows.as_array_mut().unwrap().iter_mut().find(|row|row["role"]=="resolved-supply-catalog-r3.3").unwrap();
            let text=String::from_utf8(hex::decode(row["raw_hex"].as_str().unwrap()).unwrap()).unwrap();
            let changed=if mutation=="old-schema" {text.replacen("ddn.resolved-supply-catalog.r3.3","ddn.resolved-supply-catalog.r3.2",1)} else {
                let metadata:serde_json::Value=serde_json::from_str(&text).unwrap();
                let old=metadata["pins"].as_array().unwrap().iter().find(|pin|pin["spelling"]=="내용").unwrap()["type"].as_str().unwrap();
                text.replacen(old,&format!("{old}00"),1)
            };
            assert_ne!(changed,text);
            row["raw_hex"]=hex::encode(changed.as_bytes()).into();row["byte_length"]=(changed.len() as u64).into();row["sha256"]=hex::encode(Sha256::digest(changed.as_bytes())).into();
            assert!(super::super::registration::restore_precanon_recorded_candidate(&source,serde_json::from_value(rows).unwrap()).is_err(),"{mutation}");
            assert_eq!(Source::from_artifact(&sealed).unwrap().artifact().unwrap(),sealed);
        }
    }
    #[test]
    fn r3_2_repeated_structural_dag_is_budgeted_during_expansion() {
        let mut body="(값:셈수) 층0:묶음씨 := {}. ".to_string();
        for level in 1..=22 {body.push_str(&format!("(왼쪽:층{},오른쪽:층{}) 층{level}:묶음씨 := {{}}. ",level-1,level-1));}
        body.push_str("선택:고름씨 := { $값(내용:층22). }. 실행:움직씨 := {}.");
        let artifact=project(&body);let failure=prepare(&request(&artifact,"","01",None)).unwrap_err();
        assert!(failure.contains("형식 증명 바이트 한도"),"{failure}");
    }
    #[test]
    fn vc04_workbench_source_candidate_save_update_and_readback_preserve_ids() {
        let artifact=project("명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #빨강. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let first=Source::from_artifact(&sealed).unwrap();
        assert_eq!(prepare(&request(&artifact,"01","02",Some(&sealed))).unwrap(),sealed,"unchanged save must not reissue IDs/captures");
        let mut edited=first.owned().project().clone();
        edited.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text="명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #파랑. }.".into();
        let edited=edited.save().unwrap();let next=prepare(&request(&edited,"01","02",Some(&sealed))).unwrap();let next=Source::from_artifact(&next).unwrap();
        assert_eq!(ids(&first),ids(&next));
        let before=crate::registered_atom_contract::source_binding::Candidate::parse(first.owned().registration().unwrap().descriptor()).unwrap();
        let after=crate::registered_atom_contract::source_binding::Candidate::parse(next.owned().registration().unwrap().descriptor()).unwrap();
        assert_eq!(before.registries[0].identity,after.registries[0].identity);
        for atom in &before.registries[0].entries {assert!(after.registries[0].entries.iter().any(|new|new.label==atom.label&&new.identity==atom.identity));}
        let input=serde_json::json!({"artifact_text":next.artifact().unwrap(),"project_hex":hex::encode(next.project()),"parent_hex":"01","revision_hex":"02"}).to_string();
        assert_eq!(open(&input).unwrap(),edited);
        assert!(prepare(&request(&edited,"03","04",Some(&sealed))).is_err());
    }
    #[test]
    fn vc04_workbench_source_transport_and_legacy_roles_are_separate() {
        let legacy=project("실행:움직씨 := { 1 보여주기. }.");
        assert_eq!(prepare(&request(&legacy,"","01",None)).unwrap(),"null");
        let artifact=project("명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 1 보여주기. }.");
        let sealed=prepare(&request(&artifact,"","01",None)).unwrap();let checked=Source::from_artifact(&sealed).unwrap();
        let mut raw:serde_json::Value=serde_json::from_str(&sealed).unwrap();raw["runtime_state"]="not a world".into();
        assert!(Source::from_artifact(&serde_json::to_string(&raw).unwrap()).is_err());
        let bad=serde_json::json!({"artifact_text":sealed,"project_hex":hex::encode(checked.project()),"parent_hex":"","revision_hex":"02"}).to_string();
        assert!(open(&bad).is_err());
    }
}

#[cfg(test)]
mod first_callable_product_caller_tests {
    use super::*;
    const ORIGINAL:&str = r###"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"fe6b4154262752919656f5a6f7e3629b3e81758594783b9efca19a1f0ce9ed52","modules":[{"compositions":[],"head":"세금.ddn","id":"0a9e9104a2f6a6434c2bd0dd41a721a7dacc85125c791fe47aed1deb2f4f2068","name":"세금","sources":["세금.ddn"]},{"compositions":[],"head":"app.ddn","id":"36612e8bcf97c580cb38d1c3173b719c4acbf892a937bbe97e9df9f947521c8c","name":"앱","sources":["app.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"60783cb50f9b5e604197407d2c7241c990e67171ee5608e2b10593d9b7ee1659","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"app.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\"), (이름=세금, 머리=\"세금.ddn\", 원문=\"세금.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n"},{"path":"app.ddn","text":"(세금=세금) 쓰임.\n실행:움직씨 := {\n    ((금액=(100, 1) 나눔수, 세액=(10, 1) 나눔수) 세금.총액합하다) 보여주기.\n}.\n"},{"path":"세금.ddn","text":"(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n"}],"root_path":"root.ddn"}}"###;
    const DDL:&str = r###"#!日本語
(金額:有理数~に, 税額:有理数~を)
合計する:有理数:関数 :={...}.
"###;

    fn first(project:&super::super::ProjectArtifact)->super::super::FirstCallableAuthoredContext{
        let module=project.authored_identity.modules.iter().find(|m|m.head=="세금.ddn").unwrap();
        project.first_callable_registration_context(module.id.as_str(),"세금.ddn",DDL,"세금.일본어.ddl","language-profile:ja",1).unwrap()
    }
    fn request(original:&str)->String{
        serde_json::json!({"artifact_json_utf8":original,"expected_parent_hex":"","revision_hex":"03","previous_source_artifact_utf8":null}).to_string()
    }
    fn r17_original_request(project: &super::super::ProjectArtifact) -> serde_json::Value {
        serde_json::json!({"original_project_json_utf8":project.save().unwrap(),
            "original_source_path":"세금.ddn", "ddl_source":{"path":"세금.일본어.ddl","text":DDL},
            "revision_hex":"03"})
    }
    fn r17_fresh_original() -> super::super::ProjectArtifact {
        let fixture=super::super::open(ORIGINAL.as_bytes()).unwrap();
        let created=super::super::ProjectArtifact::create(fixture.source_profile,fixture.source_snapshot,String::new()).unwrap();
        super::super::open(created.save().unwrap().as_bytes()).unwrap()
    }
    #[test]
    fn r17_original_producer_issues_actual_fresh_owner_and_export() {
        let project=r17_fresh_original();
        let original=project.save().unwrap();let context=first(&project);
        let source=super::super::operate("first_callable_original_source_prepare",&r17_original_request(&project).to_string()).unwrap();
        assert_ne!(source,"null");
        let export=context.produce_public_ddl(&original,&source).unwrap();
        assert_eq!(export.original_module_identity(),context.module_identity().as_str());
        assert_ne!(export.original_module_identity(),"0a9e9104a2f6a6434c2bd0dd41a721a7dacc85125c791fe47aed1deb2f4f2068");
        assert_ne!(export.pin_identities()[0],export.pin_identities()[1]);
        let other=r17_fresh_original();
        assert!(first(&other).produce_public_ddl(&other.save().unwrap(),&source).is_err());
    }
    #[test]
    fn r22_actual_limited_consumer_records_recheck_as_public_owner_without_reissue() {
        let project=r17_fresh_original();let context=first(&project);
        let request=r17_original_request(&project).to_string();
        let raw=super::super::operate("first_callable_original_source_prepare",&request).unwrap();
        let prepared=Source::from_artifact_with_first_callable(&raw,&context).unwrap();
        let owner=prepared.recorded_execution_owner().unwrap();let source=owner.precanon_view();
        let records=owner.registration().unwrap().recorded_blobs();
        let before=serde_json::to_vec(&records).unwrap();
        let public=super::super::registration::restore_precanon_public_callables_candidate(
            &source,context.module_identity().as_str(),records).unwrap();
        assert_eq!(serde_json::to_vec(&public.recorded_blobs()).unwrap(),before);
        let mut entropy_calls=0;
        let reused=super::super::registration::prepare_public_callables_under_candidate_owner(
            &source,context.module_identity().as_str(),Some(&public),&mut |_|{entropy_calls+=1;Err("unexpected fresh issuance".into())}).unwrap();
        assert_eq!(entropy_calls,0);assert_eq!(serde_json::to_vec(&reused.recorded_blobs()).unwrap(),before);
        let first_export=context.produce_public_ddl(&project.save().unwrap(),&raw).unwrap();
        let reopened=Source::from_artifact_with_first_callable(&raw,&first(&project)).unwrap();
        let second_export=context.produce_public_ddl(&project.save().unwrap(),&reopened.artifact().unwrap()).unwrap();
        assert_eq!(first_export.definition_identity(),second_export.definition_identity());
        assert_eq!(first_export.pin_identities(),second_export.pin_identities());
        assert_eq!(reopened.artifact().unwrap(),raw);
    }
    #[test]
    fn r17_original_producer_rejects_invalid_transport_before_registration() {
        let project=r17_fresh_original();let raw=r17_original_request(&project);
        for (field,value) in [("unexpected",serde_json::json!(true)),("revision_hex",serde_json::json!("")),
            ("original_source_path",serde_json::json!("missing.ddn")),("ddl_source",serde_json::json!({"path":"税.ddl","text":""}))] {
            let mut bad=raw.clone();bad[field]=value;
            assert!(prepare_first_callable_original(&bad.to_string()).is_err(),"{field}");
        }
        let duplicate=raw.to_string().replace("\"revision_hex\":\"03\"","\"revision_hex\":\"03\",\"revision_hex\":\"04\"");
        assert_eq!(prepare_first_callable_original(&duplicate).unwrap_err(),"E_SOURCE_SAVE_REQUEST_FIELDS");
        assert_eq!(prepare_first_callable_original(&" ".repeat(16_000_001)).unwrap_err(),"E_SOURCE_SAVE_REQUEST_LIMIT");
    }
    #[test]
    fn r17_original_producer_preserves_exact_body_scope_and_plain_prepare() {
        let original=r17_fresh_original();let mut snapshot=original.source_snapshot.clone();
        let source=snapshot.files.iter_mut().find(|f|f.path=="세금.ddn").unwrap();
        source.text=source.text.replace("금액 + 세액","금액 - 세액");
        let changed=super::super::ProjectArtifact::create(original.source_profile,snapshot,String::new()).unwrap();
        let error=prepare_first_callable_original(&r17_original_request(&changed).to_string()).unwrap_err();
        assert!(error.contains("E_DDL_FIRST_BODY_SCOPE"),"{error}");
        let unchanged=r17_fresh_original();
        assert_eq!(prepare(&request(&unchanged.save().unwrap())).unwrap(),"null");
    }
    #[test]
    fn first_callable_product_prepare_and_recorded_readback(){
        let project=super::super::open(ORIGINAL.as_bytes()).unwrap();let context=first(&project);let original=project.save().unwrap();
        let raw=super::super::operate_with_first_callable_context("source_prepare",&request(&original),&context).unwrap();
        assert_ne!(raw,"null");
        let artifact=Source::from_artifact_with_first_callable(&raw,&context).unwrap();
        let read=serde_json::json!({"artifact_text":raw,"project_hex":hex::encode(artifact.project()),"parent_hex":"","revision_hex":"03"}).to_string();
        let reopened=super::super::operate_with_first_callable_context("source_open",&read,&context).unwrap();assert_eq!(reopened,original);
        let cold=super::super::open(reopened.as_bytes()).unwrap();let fresh_context=first(&cold);
        assert_eq!(super::super::operate_with_first_callable_context("source_open",&read,&fresh_context).unwrap(),original);
        // Re-resolve and execute the unchanged R8 Japanese source against the
        // reopened actual original. This is explicit library context, not v3.
        let japanese="#!日本語\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n";
        let owner=std::sync::Arc::new(fresh_context.clone());
        let program=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(japanese,"利用.ddn",owner.clone()).unwrap();
        let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,
            last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),
            frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
        let result=crate::ddn_runtime::DdnRunner::new(program,crate::runtime_surface::DEFAULT_UPDATE_NAME)
            .run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();
        let ddonirang_lang::runtime::Value::Rational(value)=&result.resources["金額"] else{panic!("정확 나눔수 아님")};
        assert_eq!(value.ratio().numer().to_string(),"110");assert_eq!(value.ratio().denom().to_string(),"1");
        assert!(crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(
            "#!日本語\n金額 <- (金額=(100,1) 有理数) 合計する.\n","利用.ddn",owner).is_err());
        assert!(super::super::operate("source_open",&read).is_err());
        let mut same:serde_json::Value=serde_json::from_str(&request(&original)).unwrap();same["expected_parent_hex"]="03".into();same["revision_hex"]="04".into();same["previous_source_artifact_utf8"]=raw.into();
        assert_eq!(super::super::operate_with_first_callable_context("source_prepare",&same.to_string(),&fresh_context).unwrap(),serde_json::from_str::<serde_json::Value>(&read).unwrap()["artifact_text"].as_str().unwrap());
    }
    #[test]
    fn first_callable_product_recorded_payload_rejects_context_and_seal_tamper(){
        let project=super::super::open(ORIGINAL.as_bytes()).unwrap();let context=first(&project);let original=project.save().unwrap();
        let raw=super::super::operate_with_first_callable_context("source_prepare",&request(&original),&context).unwrap();
        let other=project.duplicate().unwrap();let other=super::super::open(other.save().unwrap().as_bytes()).unwrap();
        assert!(Source::from_artifact_with_first_callable(&raw,&first(&other)).is_err());
        for field in ["integrity_hex","pesi_hex","project_hex"]{
            let marker=format!("\"{field}\":\"");let start=raw.find(&marker).unwrap()+marker.len();
            let mut tampered=raw.clone();let ch=if raw.as_bytes()[start]==b'0'{"1"}else{"0"};tampered.replace_range(start..start+1,ch);
            assert!(Source::from_artifact_with_first_callable(&tampered,&context).is_err(),"{field}");
        }
        let record:serde_json::Value=serde_json::from_str(&raw).unwrap();let revision=record["revision_hex"].as_str().unwrap();assert_eq!(revision,"03");
        let reopened=Source::from_artifact_with_first_callable(&raw,&context).unwrap();
        assert_eq!(reopened.recorded_execution_owner().unwrap().project().save().unwrap(),original);
        assert_eq!(reopened.artifact().unwrap(),raw);
    }
    #[test]
    fn first_callable_product_refuses_foreign_stale_parent_and_unknown_fields(){
        let project=super::super::open(ORIGINAL.as_bytes()).unwrap();let context=first(&project);let original=project.save().unwrap();
        let other=project.duplicate().unwrap();let other=super::super::open(other.save().unwrap().as_bytes()).unwrap();
        assert!(super::super::operate_with_first_callable_context("source_prepare",&request(&other.save().unwrap()),&context).is_err());
        let mut stale=project.clone();stale.source_snapshot.files.iter_mut().find(|f|f.path=="세금.ddn").unwrap().text.push(' ');
        assert!(super::super::operate_with_first_callable_context("source_prepare",&request(&stale.save().unwrap()),&context).is_err());
        let mut extra:serde_json::Value=serde_json::from_str(&request(&original)).unwrap();extra["first_callable"]=true.into();
        assert!(super::super::operate_with_first_callable_context("source_prepare",&extra.to_string(),&context).is_err());
        assert!(super::super::operate_with_first_callable_context("run",&request(&original),&context).is_err());
        let mut parent:serde_json::Value=serde_json::from_str(&request(&original)).unwrap();parent["expected_parent_hex"]="01".into();
        assert!(super::super::operate_with_first_callable_context("source_prepare",&parent.to_string(),&context).is_err());
    }
}


#[cfg(test)]
mod r27_recorded_public_original_tests {
    use super::*;
    use crate::authored_project::{self,ProjectArtifact,SourceSnapshot,SourceFile,PublicCallableSurfaceSpec,PublicCallablePublicDdl};
    use crate::ddn_runtime::{DdnProgram,DdnRunner};
    use std::sync::Arc;
const ORIGINAL:&str=r#####"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"d93448392868869dab65fa96a8a73c3869f07eaa540229b427d60ef336fcd9e9","modules":[{"compositions":[],"head":"main.ddn","id":"5143b023273c0ce951e9b7a410db70c3f1849c2492a0ee9c46eac577a1f2350b","name":"앱","sources":["main.ddn"]},{"compositions":[],"head":"함수.ddn","id":"a21d1b41d973933d2b3957e49fe998e3a98ee6d1871bfd79a707a94f6dd2ebb4","name":"함수","sources":["함수.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"724af7c43bb6492f703355101cbeb42371f05efd4d390ff0d6a884adbdd5dd53","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"main.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=함수,머리=\"함수.ddn\",원문=\"함수.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n"},{"path":"main.ddn","text":"(함수=함수) 쓰임.\n실행:움직씨 := {\n    ((q=(3,1) 나눔수, p=(12,1) 나눔수) 함수.F) 보여주기.\n    ((r=(4,1) 나눔수) 함수.G) 보여주기.\n}.\n"},{"path":"함수.ddn","text":"(F=F, G=G) 드러냄.\n(p:나눔수~에, q:나눔수~을) F:나눔수:셈씨 := { F <- p + q. }.\n(r:나눔수~을) G:나눔수:셈씨 := { G <- r + r. }.\n"}],"root_path":"root.ddn"}}"#####;
const DDL:&str="#!日本語\n(金額:有理数~に, 税額:有理数~を) F:有理数:関数 :={...}.\n(金額:有理数~を) G:有理数:関数 :={...}.\n";
const CALL:&str="(함수=함수, 옮김=(대상말씨=\"日本語\", 대상판본=\"2\")) 쓰임.\n실행:움직씨 := {\n    ((税額=(3,1) 有理数, 金額=(12,1) 有理数) F) 보여주기.\n    ((金額=(4,1) 有理数) G) 보여주기.\n}.\n";
fn original()->ProjectArtifact{authored_project::open(ORIGINAL.as_bytes()).unwrap()}
fn module(project:&ProjectArtifact)->String{project.authored_identity.modules.iter().find(|row|row.name=="함수").unwrap().id.as_str().into()}
fn specs()->Vec<PublicCallableSurfaceSpec>{vec![
    PublicCallableSurfaceSpec{public_name:"F".into(),surface:"F".into(),pins:vec![("p".into(),"金額".into(),"に".into()),("q".into(),"税額".into(),"を".into())]},
    PublicCallableSurfaceSpec{public_name:"G".into(),surface:"G".into(),pins:vec![("r".into(),"金額".into(),"を".into())]},
]}
fn consumer(text:&str)->ProjectArtifact{
    let made=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{entry_path:"main.ddn".into(),root_path:Some("root.ddn".into()),files:vec![
        SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n".into()},
        SourceFile{path:"main.ddn".into(),text:text.into()}]},String::new()).unwrap();
    authored_project::open(made.save().unwrap().as_bytes()).unwrap()
}

    fn make_export()->PublicCallablePublicDdl {
        let original=original();original.produce_public_callable_ddl(&module(&original),specs(),DDL,"関数.ddl").unwrap()
    }
    #[test]
    fn r27_recorded_original_reconstructs_same_capture_profile_definitions_and_pins() {
        let export=make_export();let before=export.recorded_original_material().unwrap();
        let start=before.start_identity.clone();let captures=before.captures.clone();
        let blobs=serde_json::to_vec(&before.resources).unwrap();
        let profile=(before.profile.identity().to_string(),before.profile.version().to_string(),before.profile.content().to_vec());
        let restored=PublicCallablePublicDdl::restore_recorded_original(before).unwrap();
        assert_eq!(restored.facts,export.facts);
        let after=restored.recorded_original_material().unwrap();
        assert_eq!(after.start_identity,start);assert_eq!(after.captures,captures);
        assert_eq!(serde_json::to_vec(&after.resources).unwrap(),blobs);
        assert_eq!((after.profile.identity().to_string(),after.profile.version().to_string(),after.profile.content().to_vec()),profile);
        let consumer=consumer(CALL);let owner=Arc::new(restored.activate_for_consumer(&consumer,consumer.authored_identity.modules[0].id.as_str(),"main.ddn").unwrap());
        let program=DdnProgram::from_public_callable_authored_source(CALL,"main.ddn",owner).unwrap();
        let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
        let result=DdnRunner::new(program,"실행").run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();
        let actual=result.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>();
        assert_eq!(actual,["15/1","8/1"]);println!("R27_RECORDED_ORIGINAL_ACTUAL_OUTPUT={actual:?}");
    }
    #[test]
    fn r27_recorded_original_refuses_swapped_catalog_profile_and_stale_source() {
        let export=make_export();let other=make_export();
        let mut record=export.recorded_original_material().unwrap();record.profile=other.profile().clone();
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
        let mut record=export.recorded_original_material().unwrap();record.resources=other.registration.recorded_blobs();
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
        let mut record=export.recorded_original_material().unwrap();record.captures="{}".into();
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
        let mut record=export.recorded_original_material().unwrap();record.module="00".repeat(32);
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
        let mut record=export.recorded_original_material().unwrap();record.ddl=DDL.lines().take(2).collect::<Vec<_>>().join("\n");
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
        let mut record=export.recorded_original_material().unwrap();
        let original=original();let mut snapshot=original.source_snapshot.clone();
        let file=snapshot.files.iter_mut().find(|file|file.path=="함수.ddn").unwrap();file.text=file.text.replace("p + q","p + p");
        let changed=ProjectArtifact::create("v1-core-v25".into(),snapshot,String::new()).unwrap();record.original_json=changed.save().unwrap();
        assert!(PublicCallablePublicDdl::restore_recorded_original(record).is_err());
    }
    #[test]
    fn r27_existing_storage_refuses_general_consumer_without_durable_profile_record() {
        let export=make_export();let consumer=consumer(CALL);
        let restored=PublicCallablePublicDdl::restore_recorded_original(export.recorded_original_material().unwrap()).unwrap();
        restored.activate_for_consumer(&consumer,consumer.authored_identity.modules[0].id.as_str(),"main.ddn").unwrap();
        let request=serde_json::json!({"artifact_json_utf8":consumer.save().unwrap(),"expected_parent_hex":"","revision_hex":"01","previous_source_artifact_utf8":null}).to_string();
        let error=prepare(&request).unwrap_err();println!("R27_EXISTING_SOURCE_READER_REFUSAL={error}");
        // No null-success, cached-context authorization, disk write or first-profile fallback.
        assert!(!error.is_empty());
    }
}


#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicCallableOriginalPrepareV4 {
    original_project_json_utf8:String,original_source_path:String,ddl_source:super::FirstCallableDdlSource,
    surfaces:Vec<PublicCallableSurfaceV4>,revision_hex:String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicCallableSurfaceV4 {public_name:String,surface:String,pins:Vec<(String,String,String)>}
/// Original selection for Workbench. Public names and pins come from the
/// linked source; no callable registration or DDL profile is issued here.
#[cfg(windows)]
pub(crate) fn inspect_public_callable_original(
    create_request: &str,
    path: &str,
    diagnostic_out: &mut Option<crate::preprocess::ExecutionDiagnostic>,
) -> Result<String, String> {
    if create_request.len()>1_048_576{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let raw=super::operate_authoring_diagnostic("create",create_request,Some(&mut *diagnostic_out))?;
    if raw.len()>1_048_576{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let project=super::open(raw.as_bytes())?;
    let module=source_module(&project,path)?;
    let facts=super::public_surface::from_project_diagnostic(&project,module.id.as_str(),Some(&mut *diagnostic_out))?;
    if facts.declarations().is_empty(){return Err("E_DDL_ORIGINAL_PUBLIC_EMPTY".into());}
    let mut definitions=Vec::new();
    for row in facts.declarations(){
        let (seed, _) = super::public_surface::check_ddl_callable_shape(row).map_err(|error| {
            if let Some(source)=project.source_snapshot.files.iter().find(|file|file.path==row.source_path()) {
                *diagnostic_out=error.source_diagnostic(row,&source.text);
            }
            error.to_string()
        })?;
        definitions.push(serde_json::json!({"public_name":row.external_name(),
            "internal_name":row.internal_name(),"source_path":row.source_path(),
            "pins":seed.params.iter().map(|pin|pin.pin_name.as_str()).collect::<Vec<_>>() }));
    }
    Ok(serde_json::json!({"kind":"public_callable_original_inspection","module":module.name,
        "source_path":path,"definitions":definitions,"original_project_json_utf8":raw}).to_string())
}

/// Actual CLI original issuer. It produces an explicit record, never a save receipt.
pub(super) fn prepare_public_callable_original(raw:&str)->Result<String,String>{
    prepare_public_callable_original_diagnostic(raw,None)
}

pub(crate) fn prepare_public_callable_original_diagnostic(raw:&str,
    diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:PublicCallableOriginalPrepareV4=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    if request.original_project_json_utf8.len()>1_048_576||request.surfaces.is_empty()||request.surfaces.len()>128
        ||request.ddl_source.text.len()>16_384||request.surfaces.iter().any(|row|row.pins.is_empty()||row.pins.len()>32){return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    bytes(&request.revision_hex,false)?;
    let original=super::open(request.original_project_json_utf8.as_bytes())?;
    let module=source_module(&original,&request.original_source_path)?;
    let surfaces=request.surfaces.into_iter().map(|row|super::PublicCallableSurfaceSpec{
        public_name:row.public_name,surface:row.surface,pins:row.pins}).collect();
    let export=original.produce_public_callable_ddl_diagnostic(module.id.as_str(),surfaces,
        &request.ddl_source.text,&request.ddl_source.path,diagnostic)?;
    let record=super::PublicCallableOriginalRecordV4::from_export(&export,&request.original_source_path)?;
    // Roundtrip through the same exact recorded owner used by a future process;
    // no second producer/entropy call is used for the check.
    record.restore()?;record.save()
}


#[cfg(test)]
mod r28_v4_record_tests {
use super::*;
use crate::authored_project::{self,ProjectArtifact,SourceSnapshot,SourceFile,PublicCallableSurfaceSpec,PublicCallableProjectArtifactV4,PublicCallableOriginalRecordV4};
use crate::ddn_runtime::{DdnProgram,DdnRunner};
use std::sync::Arc;
const ORIGINAL:&str=r#####"{"authored_identity":{"clone_origin":null,"entry":null,"logical_dependency_id":"d93448392868869dab65fa96a8a73c3869f07eaa540229b427d60ef336fcd9e9","modules":[{"compositions":[],"head":"main.ddn","id":"5143b023273c0ce951e9b7a410db70c3f1849c2492a0ee9c46eac577a1f2350b","name":"앱","sources":["main.ddn"]},{"compositions":[],"head":"함수.ddn","id":"a21d1b41d973933d2b3957e49fe998e3a98ee6d1871bfd79a707a94f6dd2ebb4","name":"함수","sources":["함수.ddn"]}],"schema":"ddn.authored_identity.v1"},"content_sha256":"724af7c43bb6492f703355101cbeb42371f05efd4d390ff0d6a884adbdd5dd53","schema":"ddn.authored_project_artifact.v1","source_profile":"v1-core-v25","source_snapshot":{"entry_path":"main.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=함수,머리=\"함수.ddn\",원문=\"함수.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n"},{"path":"main.ddn","text":"(함수=함수) 쓰임.\n실행:움직씨 := {\n    ((q=(3,1) 나눔수, p=(12,1) 나눔수) 함수.F) 보여주기.\n    ((r=(4,1) 나눔수) 함수.G) 보여주기.\n}.\n"},{"path":"함수.ddn","text":"(F=F, G=G) 드러냄.\n(p:나눔수~에, q:나눔수~을) F:나눔수:셈씨 := { F <- p + q. }.\n(r:나눔수~을) G:나눔수:셈씨 := { G <- r + r. }.\n"}],"root_path":"root.ddn"}}"#####;
const DDL:&str="#!日本語\n(金額:有理数~に, 税額:有理数~を) F:有理数:関数 :={...}.\n(金額:有理数~を) G:有理数:関数 :={...}.\n";
const CALL:&str="(함수=함수, 옮김=(대상말씨=\"日本語\", 대상판본=\"2\")) 쓰임.\n실행:움직씨 := {\n    ((税額=(3,1) 有理数, 金額=(12,1) 有理数) F) 보여주기.\n    ((金額=(4,1) 有理数) G) 보여주기.\n}.\n";
fn original()->ProjectArtifact{authored_project::open(ORIGINAL.as_bytes()).unwrap()}
fn module(project:&ProjectArtifact)->String{project.authored_identity.modules.iter().find(|row|row.name=="함수").unwrap().id.as_str().into()}
fn specs()->Vec<PublicCallableSurfaceSpec>{vec![
    PublicCallableSurfaceSpec{public_name:"F".into(),surface:"F".into(),pins:vec![("p".into(),"金額".into(),"に".into()),("q".into(),"税額".into(),"を".into())]},
    PublicCallableSurfaceSpec{public_name:"G".into(),surface:"G".into(),pins:vec![("r".into(),"金額".into(),"を".into())]},
]}
fn consumer(text:&str)->ProjectArtifact{
    let made=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{entry_path:"main.ddn".into(),root_path:Some("root.ddn".into()),files:vec![
        SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n".into()},
        SourceFile{path:"main.ddn".into(),text:text.into()}]},String::new()).unwrap();
    authored_project::open(made.save().unwrap().as_bytes()).unwrap()
}

fn record()->PublicCallableOriginalRecordV4{
 let project=original();let export=project.produce_public_callable_ddl(&module(&project),specs(),DDL,"関数.ddl").unwrap();
 PublicCallableOriginalRecordV4::from_export(&export,"함수.ddn").unwrap()
}
#[test]
fn r28_record_bytes_restore_new_registry_owner_without_reissuing_profile(){
 let record=record();let raw=record.save().unwrap();let before=record.restore().unwrap();
 let cold=PublicCallableOriginalRecordV4::open(&raw).unwrap().restore().unwrap();
 assert_eq!(cold.facts,before.facts);assert_eq!(cold.profile().identity(),before.profile().identity());
 assert_eq!(cold.profile().content(),before.profile().content());
 assert_eq!(cold.recorded_original_material().unwrap().captures,before.recorded_original_material().unwrap().captures);
 let mut owner=ddonirang_lang::ddl::SuccessorRegistryOwner::default();let profile=before.profile();
 let content=std::str::from_utf8(profile.content()).unwrap();
 owner.restore_recorded_exact(profile.identity(),profile.version(),profile.registration_revision(),content,profile.content_sha256()).unwrap();
 assert!(owner.restore_recorded_exact("language-profile:11111111111111111111111111111111",profile.version(),profile.registration_revision(),content,profile.content_sha256()).is_err());
 assert!(ddonirang_lang::ddl::SuccessorRegistryOwner::default().restore_recorded_exact(profile.identity(),"1",1,content,profile.content_sha256()).is_err());
 println!("R28_RECORD_FRESH_OWNER_SAME_PROFILE={}",cold.profile().identity());
}
#[test]
fn r245_spelling_only_edition_preserves_original_callable_and_pin_owners(){
 let previous=record();
 let before=previous.save().unwrap();
 let old=previous.restore().unwrap();
 let mut surfaces=specs();
 surfaces[0].surface="NewF".into();
 surfaces[0].pins[0].1="Amount".into();
 let ddl=DDL.replace("金額:有理数~に","Amount:有理数~に").replace(") F:",") NewF:");
 let revised=PublicCallableOriginalRecordV4::open(&before).unwrap().revise_surfaces(
     "3",&surfaces,super::super::FirstCallableDdlSource{path:"関数.ddl".into(),text:ddl.clone()},None).unwrap();
 let draft=revised.save().unwrap();
 assert_eq!(PublicCallableOriginalRecordV4::open(&draft).err().as_deref(),Some("E_DDL_EDITION_HISTORY_REQUIRED"));
 let cold=super::super::public_read::<PublicCallableOriginalRecordV4>(&draft).unwrap().restore_edition_draft().unwrap();
 assert_eq!(cold.profile().identity(),old.profile().identity());
 assert_eq!(cold.profile().version(),"3");
 assert_eq!(cold.profile().registration_revision(),2);
 assert_eq!(cold.facts,old.facts);
 assert_eq!(revised.original_project_json_utf8,previous.original_project_json_utf8);
 assert_eq!(revised.original_source_capture_catalog_json_utf8,previous.original_source_capture_catalog_json_utf8);
 assert_eq!(previous.save().unwrap(),before);
 assert_eq!(PublicCallableOriginalRecordV4::open(&before).unwrap().restore().unwrap().profile().version(),"2");
 let good=consumer(&CALL.replace("대상판본=\"2\"","대상판본=\"3\"")
     .replace("金額=(12","Amount=(12").replace(") F)",") NewF)"));
 let owner=good.authored_identity.modules.iter().find(|m|m.sources.iter().any(|path|path=="main.ddn")).unwrap();
 assert!(cold.activate_for_consumer(&good,owner.id.as_str(),"main.ddn").is_ok());
 assert!(PublicCallableProjectArtifactV4::create(revised,&good,"main.ddn",None).is_err());
 let source=||super::super::FirstCallableDdlSource{path:"関数.ddl".into(),text:ddl.clone()};
 assert!(PublicCallableOriginalRecordV4::open(&before).unwrap().revise_surfaces("4",&surfaces,source(),None).is_err());
 let mut wrong=surfaces;
 wrong[0].pins[0].0="r".into();
 assert!(PublicCallableOriginalRecordV4::open(&before).unwrap().revise_surfaces("3",&wrong,source(),None).is_err());
}
#[cfg(windows)]
#[test]
fn r246_public_edition_draft_requires_exact_native_parent_and_same_consumer_meaning(){
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
    let mut nonce=[0u8;8];
    getrandom::fill(&mut nonce).unwrap();
    let dir=std::env::temp_dir().join(format!("ddl246-{}",hex::encode(nonce)));
    std::fs::create_dir(&dir).unwrap();
    let old=PublicCallableProjectArtifactV4::create(record(),&consumer(CALL),"main.ddn",None).unwrap();
    let owned=crate::project_revision_envelope_supply::admit_fresh_public_callable_project(old.clone()).unwrap();
    let previous=Source::prepare(&owned,&[],b"original").unwrap();
    let mut adapter=SqliteAdapterV1::provision_source_revision(&dir).unwrap();
    adapter.save_initial_source_revision(&previous).unwrap();
    drop(adapter);
    let adapter=SqliteAdapterV1::open_source_revision(&dir).unwrap();

    let mut surfaces=specs();
    surfaces[0].surface="NewF".into();
    surfaces[0].pins[0].1="Amount".into();
    let ddl=||super::super::FirstCallableDdlSource{path:"関数.ddl".into(),
        text:DDL.replace("金額:有理数~に","Amount:有理数~に").replace(") F:",") NewF:")};
    let next=CALL.replace("대상판본=\"2\"","대상판본=\"3\"")
        .replace("金額=(12","Amount=(12").replace(") F)",") NewF)");
    let prepare=|expected:&Source,revision:&[u8],text:String|
        prepare_public_edition_from_current(&adapter,expected,revision,"3",&surfaces,ddl(),text);
    let draft=prepare(&previous,b"successor",next.clone()).unwrap();
    assert_eq!(draft.parent.artifact().unwrap(),previous.artifact().unwrap());
    assert_eq!(draft.revision,b"successor");
    assert_eq!(draft.artifact.authored_identity,old.authored_identity);
    assert_eq!(draft.artifact.source_snapshot.root_path,old.source_snapshot.root_path);
    assert_eq!(draft.artifact.source_snapshot.entry_path,old.source_snapshot.entry_path);
    assert_eq!(draft.artifact.source_snapshot.files.iter().filter(|file|file.path!="main.ddn").collect::<Vec<_>>(),
        old.source_snapshot.files.iter().filter(|file|file.path!="main.ddn").collect::<Vec<_>>());
    assert_eq!(draft.artifact.restore_context().err().as_deref(),Some("E_DDL_EDITION_HISTORY_REQUIRED"));

    let stale=Source::prepare(&owned,&[],b"not-current").unwrap();
    assert!(prepare(&stale,b"successor",next.clone()).err().unwrap().starts_with("E_DDL_EDITION_PARENT:"));
    let mut edited=old.source_view().unwrap();
    let file=edited.source_snapshot.files.iter_mut().find(|file|file.path=="main.ddn").unwrap();
    file.text=file.text.replace("(12,1)","(13,1)");
    edited.content_sha256=edited.checksum().unwrap();
    let alternate=PublicCallableProjectArtifactV4::create(old.original_record().unwrap(),&edited,"main.ddn",None).unwrap();
    let alternate=crate::project_revision_envelope_supply::admit_fresh_public_callable_project(alternate).unwrap();
    let wrong_integrity=Source::prepare(&alternate,&[],b"original").unwrap();
    assert_eq!(wrong_integrity.project(),previous.project());
    assert_ne!(wrong_integrity.integrity(),previous.integrity());
    assert!(prepare(&wrong_integrity,b"successor",next.clone()).err().unwrap().starts_with("E_DDL_EDITION_PARENT:"));
    assert_eq!(prepare(&previous,b"original",next.clone()).err().as_deref(),Some("E_SOURCE_REVISION_IDENTITY"));
    assert_eq!(prepare(&previous,b"successor",next.replace("(12,1)","(13,1)")).err().as_deref(),
        Some("E_DDL_EDITION_CONSUMER_MEANING_CHANGED"));
    let saved=adapter.load_current_source_revision(previous.project(),previous.revision(),previous.integrity()).unwrap();
    assert_eq!(saved.source().artifact().unwrap(),previous.artifact().unwrap());
}
#[cfg(windows)]
#[test]
fn r247_public_edition_cas_and_durable_parent_child_readback(){
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
    let mut nonce=[0u8;8];
    getrandom::fill(&mut nonce).unwrap();
    let dir=std::env::temp_dir().join(format!("ddl247-{}",hex::encode(nonce)));
    std::fs::create_dir(&dir).unwrap();
    let old=PublicCallableProjectArtifactV4::create(record(),&consumer(CALL),"main.ddn",None).unwrap();
    let owned=crate::project_revision_envelope_supply::admit_fresh_public_callable_project(old).unwrap();
    let previous=Source::prepare(&owned,&[],b"original").unwrap();
    let mut adapter=SqliteAdapterV1::provision_source_revision(&dir).unwrap();
    adapter.save_initial_source_revision(&previous).unwrap();
    let mut surfaces=specs();
    surfaces[0].surface="NewF".into();
    surfaces[0].pins[0].1="Amount".into();
    let ddl=||super::super::FirstCallableDdlSource{path:"関数.ddl".into(),
        text:DDL.replace("金額:有理数~に","Amount:有理数~に").replace(") F:",") NewF:")};
    let next=CALL.replace("대상판본=\"2\"","대상판본=\"3\"")
        .replace("金額=(12","Amount=(12").replace(") F)",") NewF)");
    let prepare=|adapter:&SqliteAdapterV1,rev:&[u8]|
        prepare_public_edition_from_current(adapter,&previous,rev,"3",&surfaces,ddl(),next.clone()).unwrap();
    let stale=prepare(&adapter,b"stale");
    let child=prepare(&adapter,b"successor").save(&mut adapter).unwrap();
    assert!(stale.save(&mut adapter).is_err());
    assert!(Source::from_artifact(&child.artifact().unwrap()).is_err());
    drop(adapter);
    let adapter=SqliteAdapterV1::open_source_revision(&dir).unwrap();
    let (reopened_old,reopened_child)=adapter.read_public_edition_history(&previous,child.revision(),child.integrity()).unwrap();
    assert_eq!(reopened_old.artifact().unwrap(),previous.artifact().unwrap());
    assert_eq!(reopened_child.artifact().unwrap(),child.artifact().unwrap());
    assert_eq!(reopened_child.project(),reopened_old.project());
    assert_eq!(reopened_child.parent(),reopened_old.revision());
    let before=reopened_old.recorded_execution_owner().unwrap();
    let after=reopened_child.recorded_execution_owner().unwrap();
    let context_before=before.sources().public_callable_context().unwrap();
    let context_after=after.sources().public_callable_context().unwrap();
    assert_eq!(context_before.definition_identities(),context_after.definition_identities());
    assert_eq!(context_before.export.facts,context_after.export.facts);
    assert_eq!(context_before.compiler_context().profile().version(),"2");
    assert_eq!(context_after.compiler_context().profile().version(),"3");
}
#[test]
fn r28_v4_reader_preserves_raw_source_and_common_runtime_results(){
 let consumer=consumer(CALL);let artifact=PublicCallableProjectArtifactV4::create(record(),&consumer,"main.ddn",None).unwrap();
 let raw=artifact.save().unwrap();let cold=PublicCallableProjectArtifactV4::open(&raw).unwrap();
 assert_eq!(cold.source_snapshot,consumer.source_snapshot);assert_eq!(cold.authored_identity,consumer.authored_identity);
 assert_eq!(cold.dependency_material().unwrap(),artifact.dependency_material().unwrap());
 let context=cold.restore_context().unwrap();let program=DdnProgram::from_public_callable_authored_source(CALL,"main.ddn",context).unwrap();
 let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
 let result=DdnRunner::new(program,"실행").run_update(&ddonirang_core::NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();
 let texts=result.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>();assert_eq!(texts,["15/1","8/1"]);
 assert!(authored_project::open(raw.as_bytes()).is_err(),"legacy reader must refuse v4");
 assert_eq!(original().save().unwrap(),ORIGINAL,"legacy v1 exact bytes");
 println!("R28_V4_COLD_TYPED_READER_ACTUAL_OUTPUT={texts:?}; NOT_NATIVE_COLD");
}
#[test]
fn r28_v4_rejects_profile_owner_and_mixed_wire_tampering(){
 let consumer=consumer(CALL);let artifact=PublicCallableProjectArtifactV4::create(record(),&consumer,"main.ddn",None).unwrap();let raw=artifact.save().unwrap();
 let mut record=record();record.profile.content_sha256="00".repeat(32);assert!(PublicCallableOriginalRecordV4::open(&record.save().unwrap()).is_err());
 let mut value:serde_json::Value=serde_json::from_str(&raw).unwrap();value["public_callable_binding"]["consumer_module_id"]="00".repeat(32).into();
 let mut altered:PublicCallableProjectArtifactV4=serde_json::from_value(value).unwrap();altered.content_sha256=altered.checksum().unwrap();
 assert!(PublicCallableProjectArtifactV4::open(&altered.save().unwrap()).is_err(),"correct checksum must not grant owner authority");
 let mixed=raw.replacen("{","{\"language_binding\":null,",1);assert!(PublicCallableProjectArtifactV4::open(&mixed).is_err());
 let duplicate=raw.replacen("{","{\"schema\":\"ddn.authored_project_artifact.v4\",",1);assert!(PublicCallableProjectArtifactV4::open(&duplicate).is_err());
}
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicCallablePrepareV4 {original_record_json_utf8:String,consumer_project_json_utf8:String,consumer_source_path:String,revision_hex:String}
pub(super) fn prepare_public_callable_product(raw:&str,
    mut diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:PublicCallablePrepareV4=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let revision=bytes(&request.revision_hex,false)?;
    let original=super::PublicCallableOriginalRecordV4::open(&request.original_record_json_utf8)?;
    let consumer=super::open(request.consumer_project_json_utf8.as_bytes())?;
    let artifact=super::PublicCallableProjectArtifactV4::create(original,&consumer,&request.consumer_source_path,diagnostic.as_deref_mut())?;
    let owned=crate::project_revision_envelope_supply::admit_fresh_public_callable_project_diagnostic(artifact,diagnostic)?;
    Source::prepare(&owned,&[],&revision)?.artifact()
}
pub(super) fn run_public_callable_product(raw:&str)->Result<String,String>{
    if logic_consumer::public_run_is_selected(raw)?{return logic_consumer::operate_private("public_callable_source_run",raw);}
    run_v4_public_callable_product(raw, None)
}

pub(super) fn run_v4_public_callable_product(
    raw: &str,
    diagnostic_out: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
) -> Result<String, String> {
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:FirstCallableRun=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=Source::from_artifact(&request.source_artifact_utf8)?;
    run_public_source_with_diagnostic(&source, diagnostic_out, false)
}

pub(crate) fn run_public_source(source:&Source)->Result<String,String>{
    run_public_source_diagnostic(source, None)
}

pub(crate) fn run_public_source_diagnostic(
    source: &Source,
    diagnostic_out: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
) -> Result<String, String> {
    run_public_source_with_diagnostic(source, diagnostic_out, true)
}

fn run_public_source_with_diagnostic(
    source: &Source,
    diagnostic_out: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    locate_zero_denominator: bool,
) -> Result<String, String> {
    let owned=source.recorded_execution_owner()?;
    let public=owned.sources().public_callable_context().ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?;
    let artifact=owned.sources().public_callable_artifact().ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?;
    let path=artifact.consumer_path();let file=artifact.source_snapshot.files.iter().find(|f|f.path==path).ok_or("E_DDL_CONSUMER_SOURCE_REQUIRED")?;
    let program=crate::ddn_runtime::DdnProgram::from_public_callable_authored_source(&file.text,path,public.clone())?;
    let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
    let result = crate::ddn_runtime::DdnRunner::new(program, owned.sources().entry_declaration().name())
        .run_update_diagnostic(&ddonirang_core::NuriWorld::new(), &input, &std::collections::HashMap::new())
        .map_err(|error| {
            use crate::ddn_runtime::EvalError;
            if let EvalError::SourceLocated { error: cause, span: (start, end), .. } = &error {
                let zero_denominator = matches!(cause.as_ref(), EvalError::RationalZeroDenominator);
                if zero_denominator || matches!(cause.as_ref(), EvalError::DivisionByZero
                    | EvalError::ArithmeticFault(ddonirang_core::ArithmeticFaultKind::DivByZero))
                {
                    let message = cause.to_string();
                    if let Some(output) = diagnostic_out.filter(|_| !zero_denominator || locate_zero_denominator) {
                        // This runner parsed the exact checked consumer source, not a linked projection.
                        if start < end && file.text.get(*start..*end).is_some() {
                            let mut failure = crate::runtime_surface::execution_error_diagnostic(error, path);
                            failure.message.clone_from(&message);
                            *output = Some(failure);
                        }
                    }
                    return message;
                }
            }
            error.to_string()
        })?;
    serde_json::to_string(&serde_json::json!({"output_log_texts":result.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>(),
        "source_start_identity_hex":hex::encode(owned.definition_identity().as_bytes()),"original_binding":artifact.dependency_material()?,"source_only":true})).map_err(|_|"E_MULTILANG_RESULT_ENCODING".into())
}
/// A verified parent and draft. The transaction compares this parent's
/// revision and integrity again before committing the edition.
#[cfg(windows)]
pub(crate) struct PublicEditionCandidate {
    parent: Source,
    artifact: super::PublicCallableProjectArtifactV4,
}

#[cfg(windows)]
impl PublicEditionCandidate {
    pub(crate) fn review_hash(&self)->Result<String,String>{
        use sha2::{Digest,Sha256};
        // Parent integrity binds the project and exact parent revision.
        let mut hash=Sha256::new();
        hash.update(b"ddn.workbench.public_edition_review.v1");
        hash.update(self.parent.integrity());
        hash.update(self.artifact.save()?.as_bytes());
        Ok(hex::encode(hash.finalize()))
    }

    pub(crate) fn prepare(self,revision:&[u8])->Result<PublicEditionDraft,String>{
        if revision.is_empty()||revision.len()>4096||revision==self.parent.revision(){
            return Err("E_SOURCE_REVISION_IDENTITY".into());
        }
        Ok(PublicEditionDraft{parent:self.parent,revision:revision.to_vec(),artifact:self.artifact})
    }
}

#[cfg(windows)]
pub(crate) struct PublicEditionDraft {
    parent: Source,
    revision: Vec<u8>,
    artifact: super::PublicCallableProjectArtifactV4,
}

#[cfg(windows)]
impl PublicEditionDraft {
    pub(crate) fn save(self,adapter:&mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)->Result<Source,String>{
        let previous=self.parent.owned().sources().public_callable_artifact().ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?;
        let artifact=super::PublicCallableProjectArtifactV4::open_spelling_successor(previous,&self.artifact.save()?)?;
        let owned=crate::project_revision_envelope_supply::admit_fresh_public_callable_project(artifact)?;
        let source=Source::prepare(&owned,self.parent.revision(),&self.revision)?;
        adapter.save_public_edition_successor(&self.parent,&source).map_err(|e|format!("E_DDL_EDITION_SAVE: {e:?}"))
    }
}

#[cfg(windows)]
fn prepare_public_edition_from_current(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    expected:&Source, revision:&[u8], version:&str, surfaces:&[super::PublicCallableSurfaceSpec],
    ddl:super::FirstCallableDdlSource, consumer_text:String,
)->Result<PublicEditionDraft,String>{
    if revision.is_empty()||revision.len()>4096||revision==expected.revision(){
        return Err("E_SOURCE_REVISION_IDENTITY".into());
    }
    check_public_edition_from_current(adapter,expected,version,surfaces,ddl,consumer_text,None)?.prepare(revision)
}

#[cfg(windows)]
pub(crate) fn check_public_edition_from_current(
    adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    expected:&Source, version:&str, surfaces:&[super::PublicCallableSurfaceSpec],
    ddl:super::FirstCallableDdlSource, consumer_text:String,
    diagnostic:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
)->Result<PublicEditionCandidate,String>{
    let stored = match adapter.load_current_source_revision(
        expected.project(), expected.revision(), expected.integrity(),
    ) {
        Ok(stored) => stored,
        Err(error) => {
            // Report a stale parent only after the retained pair and its head
            // have been verified. A damaged or unsupported store keeps its error.
            if let Ok((initial, Some(_))) = adapter.read_public_editions_at_anchor(
                expected.project(), expected.revision(), expected.integrity(),
            ) {
                if initial.artifact().is_ok_and(|artifact| {
                    expected.artifact().is_ok_and(|expected| artifact == expected)
                }) {
                    return Err("E_DDL_EDITION_PARENT_MISMATCH".into());
                }
            }
            return Err(format!("E_DDL_EDITION_PARENT: {error:?}"));
        }
    };
    let parent=stored.source();
    if parent.artifact()?!=expected.artifact()?{return Err("E_DDL_EDITION_PARENT_MISMATCH".into());}
    let artifact=parent.owned().sources().public_callable_artifact()
        .ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?
        .check_spelling_edition(version,surfaces,ddl,consumer_text,diagnostic)?;
    Ok(PublicEditionCandidate{parent:parent.clone(),artifact})
}

/// Private native author-tool transport. The ordinary public callable
/// operations and their first-save wire keep the existing behavior.
#[cfg(windows)]
pub(super) fn operate_public_edition_native(operation:&str,raw:&str)->Result<String,String>{
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Save {
        storage_root:std::path::PathBuf, parent_source_artifact_utf8:String, revision_hex:String,
        edition:String, surfaces:Vec<PublicCallableSurfaceV4>, ddl_source:super::FirstCallableDdlSource,
        consumer_source_utf8:String,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Read {
        storage_root:std::path::PathBuf, parent_source_artifact_utf8:String,
        head_revision_hex:String, head_integrity_hex:String, selected_revision_hex:String,
    }
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    if operation=="public_callable_edition_update_native"{
        let request:Save=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
        let previous=Source::from_artifact(&request.parent_source_artifact_utf8)?;
        let revision=bytes(&request.revision_hex,false)?;
        let surfaces=request.surfaces.into_iter().map(|row|super::PublicCallableSurfaceSpec{
            public_name:row.public_name,surface:row.surface,pins:row.pins,
        }).collect::<Vec<_>>();
        let mut adapter=SqliteAdapterV1::open_source_revision(&request.storage_root)
            .map_err(|error|format!("E_SOURCE_REVISION_OPEN: {error:?}"))?;
        let draft=prepare_public_edition_from_current(&adapter,&previous,&revision,&request.edition,
            &surfaces,request.ddl_source,request.consumer_source_utf8)?;
        let saved=draft.save(&mut adapter)?;
        return serde_json::to_string(&serde_json::json!({
            "head_revision_hex":hex::encode(saved.revision()),
            "head_integrity_hex":hex::encode(saved.integrity()),
        })).map_err(|_|"E_DDL_EDITION_RECEIPT".into());
    }
    let request:Read=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let previous=Source::from_artifact(&request.parent_source_artifact_utf8)?;
    let head=bytes(&request.head_revision_hex,false)?;
    let integrity=bytes(&request.head_integrity_hex,false)?.try_into().map_err(|_|"E_SOURCE_ARTIFACT_INTEGRITY")?;
    let selected=bytes(&request.selected_revision_hex,false)?;
    if selected!=previous.revision()&&selected!=head{return Err("E_DDL_EDITION_REVISION_SELECTION".into());}
    let adapter=SqliteAdapterV1::open_source_revision(&request.storage_root)
        .map_err(|error|format!("E_SOURCE_REVISION_OPEN: {error:?}"))?;
    let (old,new)=adapter.read_public_edition_history(&previous,&head,&integrity)
        .map_err(|error|format!("E_DDL_EDITION_READBACK: {error:?}"))?;
    let source=if selected==old.revision(){old}else{new};
    match operation {
        "public_callable_edition_reopen_native"=>source.artifact(),
        "public_callable_edition_run_native"=>run_public_source(&source),
        _=>Err("E_DDL_EDITION_OPERATION".into()),
    }
}
#[cfg(not(windows))]
pub(super) fn operate_public_edition_native(_:&str,_:&str)->Result<String,String>{
    Err("E_MULTILANG_NATIVE_UNSUPPORTED".into())
}

#[cfg(not(target_arch="wasm32"))]
fn public_native_request(raw:&str)->Result<(FirstCallableNative,Source),String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let strict:super::StrictValue=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let request:FirstCallableNative=serde_json::from_value(strict.0).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=Source::from_artifact(&request.source_artifact_utf8)?;
    if source.owned().sources().public_callable_artifact().is_none(){return Err("E_DDL_V4_RECORDED_CONTEXT_REQUIRED".into());}
    Ok((request,source))
}
#[cfg(not(target_arch="wasm32"))]
pub(super) fn save_public_callable_native(raw:&str)->Result<String,String>{
    if logic_consumer::public_native_is_selected(raw)?{return logic_consumer::operate_private("public_callable_source_save_native",raw);}
    use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,source_revision::SourceSaveFailure};
    let (request,source)=public_native_request(raw)?;
    if !source.parent().is_empty(){return Err("E_SOURCE_SAVE_PARENT: 첫 원문 저장에는 빈 부모만 허용합니다".into());}
    let mut adapter=SqliteAdapterV1::provision_source_revision(&request.storage_root).map_err(|e|format!("E_SOURCE_SAVE_BEFORE_COMMIT: {e:?}"))?;
    let read=adapter.save_initial_source_revision(&source).map_err(|e|match e{
        SourceSaveFailure::BeforeCommit(cause)=>format!("E_SOURCE_SAVE_BEFORE_COMMIT: {cause:?}"),
        SourceSaveFailure::AfterCommitUnconfirmed(cause)=>format!("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 뒤 확인되지 않았습니다 ({cause:?})"),
    })?;
    let actual=read.source().artifact()?;
    if actual!=request.source_artifact_utf8{return Err("E_SOURCE_SAVE_UNCONFIRMED: RecoveryRequired: 저장 후 원문이 다릅니다".into());}Ok(actual)
}
#[cfg(not(target_arch="wasm32"))]
pub(super) fn reopen_public_callable_native(raw:&str)->Result<String,String>{
    if logic_consumer::public_native_is_selected(raw)?{return logic_consumer::operate_private("public_callable_source_reopen_native",raw);}
    let (request,expected)=public_native_request(raw)?;
    let adapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_source_revision(&request.storage_root).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let read=adapter.load_current_source_revision(expected.project(),expected.revision(),expected.integrity()).map_err(|e|format!("E_SOURCE_REVISION_READBACK: {e:?}"))?;
    let actual=read.source().artifact()?;
    if actual!=request.source_artifact_utf8{return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 저장된 전체 원문·정체와 기대 문맥이 다릅니다".into());}Ok(actual)
}
#[cfg(target_arch="wasm32")]
pub(super) fn save_public_callable_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED".into())}
#[cfg(target_arch="wasm32")]
pub(super) fn reopen_public_callable_native(_: &str)->Result<String,String>{Err("E_MULTILANG_NATIVE_UNSUPPORTED".into())}

#[cfg(test)]
mod current_model_source_storage_tests {
    use super::*;
    const SOURCE:&str=include_str!("../../tests/fixtures/model_current_surface.또니");
    fn artifact(source:&str)->String {
        crate::authored_project::ProjectArtifact::create("v1-core-v25".into(),
            crate::authored_project::SourceSnapshot{root_path:None,
                entry_path:"관계표원문.또니".into(),files:vec![crate::authored_project::SourceFile{
                    path:"관계표원문.또니".into(),text:source.into()}]},"모형학습".into())
            .unwrap().save().unwrap()
    }
    #[test]
    fn current_model_source_probe_uses_the_same_profile_and_actual_model_parser() {
        let raw=artifact(SOURCE);
        let probe=serde_json::json!({"artifact_json_utf8":raw}).to_string();
        assert_eq!(crate::authored_project::operate("source_probe",&probe).unwrap(),"false");
        let request=serde_json::json!({"artifact_json_utf8":raw,
            "expected_parent_hex":"","revision_hex":"01","previous_source_artifact_utf8":null}).to_string();
        // Preserve the existing source-only selection; do not manufacture a
        // Native root/source envelope or run a world during classification.
        assert_eq!(crate::authored_project::operate("source_prepare",&request).unwrap(),"null");
        let reopened=crate::authored_project::open(raw.as_bytes()).unwrap();
        assert_eq!(reopened.source_snapshot.files[0].text,SOURCE);
        assert_eq!(reopened.save().unwrap(),raw);
        assert!(!checked_standalone_model("v1-core","관계표원문.또니",SOURCE).unwrap());
        assert!(!checked_standalone_model("v1-core-v25","일반.또니","값 := 3.").unwrap());
    }
    #[test]
    fn current_model_source_probe_keeps_model_syntax_and_relation_rejections() {
        for source in [SOURCE.replace("가속도 =:= 힘 / 질량.",""),
            SOURCE.replace("질량:수@kg","질량:수@없는단위"),
            SOURCE.replace("힘 =:= -상수 * 위치.","힘 =:= .")] {
            let raw=artifact(&source);
            let request=serde_json::json!({"artifact_json_utf8":raw}).to_string();
            let error=crate::authored_project::operate("source_probe",&request).unwrap_err();
            assert!(error.contains("E_MODEL_"),"{error}");
            assert!(!error.contains("E_V25_DEFINITION_MIGRATION_REQUIRED"),"{error}");
        }
    }
}
