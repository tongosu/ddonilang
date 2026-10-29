//! Workbench source-only candidate transport. Actual strict IndexedDB CAS and
//! full recorded readback follow this authoring step; it cannot start a world.
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
fn nominal(project:&super::ProjectArtifact)->Result<bool,String> {
    let mut found=false;
    for module in &project.authored_identity.modules {for path in &module.sources {
        let source=project.source_snapshot.files.iter().find(|file|file.path==*path).ok_or("E_SOURCE_SAVE_MODULE_SOURCE")?;
        let parsed=ddonirang_lang::parse_v25_with_mode(&source.text,path,ddonirang_lang::ParseMode::Strict).map_err(|e|e.to_string())?;
        found|=parsed.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)));
        found|=super::registration::requires_resolved_program(&parsed);
    }}
    Ok(found)
}
pub(super) fn requirements(raw:&str)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Probe=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    Ok(nominal(&super::open(request.artifact_json_utf8.as_bytes())?)?.to_string())
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

pub(super) fn prepare(raw:&str)->Result<String,String>{prepare_selected(raw,None)}
pub(super) fn prepare_first_callable(raw:&str,first:&super::FirstCallableAuthoredContext)->Result<String,String>{
    prepare_selected(raw,Some(first))
}
fn restore_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>)->Result<Source,String>{
    match first {Some(first)=>Source::from_artifact_with_first_callable(raw,first),None=>Source::from_artifact(raw)}
}
fn prepare_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Prepare=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let parent=bytes(&request.expected_parent_hex,true)?;let revision=bytes(&request.revision_hex,false)?;
    let project=super::open(request.artifact_json_utf8.as_bytes())?;
    let v3=project.schema==super::SCHEMA_V3;
    if v3 {first.ok_or("E_DDL_V3_RECORDED_CONTEXT_REQUIRED")?.registration_record_for_consumer(&project)?;}
    // Preserve established authoring/runtime workflows while VC04's source
    // owner is wired. This new UI route is nominal roots + their retained
    // catalogs only; it is not a claim that every source-save route is closed.
    if first.is_none()&&!nominal(&project)?&&request.previous_source_artifact_utf8.is_none(){return Ok("null".into());}
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
        Some(first)=>super::registration::prepare_first_callable_under_candidate_owner(&source,first,old,&mut entropy)?,
        None=>super::registration::prepare_precanon_under_candidate_owner(&source,old,&mut entropy)?,
    };
    let owned=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft)?;
    Source::prepare(&owned,&parent,&revision)?.artifact()
}
pub(super) fn open(raw:&str)->Result<String,String>{open_selected(raw,None)}
pub(super) fn open_first_callable(raw:&str,first:&super::FirstCallableAuthoredContext)->Result<String,String>{open_selected(raw,Some(first))}
fn open_selected(raw:&str,first:Option<&super::FirstCallableAuthoredContext>)->Result<String,String>{
    if raw.len()>16_000_000{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let request:Open=serde_json::from_str(raw).map_err(|_|"E_SOURCE_SAVE_REQUEST_FIELDS")?;
    let source=restore_selected(&request.artifact_text,first)?;
    if source.project()!=bytes(&request.project_hex,false)?||source.parent()!=bytes(&request.parent_hex,true)?||source.revision()!=bytes(&request.revision_hex,false)? {
        return Err("E_SOURCE_SAVE_READBACK_CONTEXT: 원문 저장본과 실제 저장 머리가 다릅니다".into());
    }
    source.owned().project().save()
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
