//! Physical same-origin Native host transport. JSON is only a locator/request;
//! all execution authority stays in the protected Native source owner.
use std::{collections::BTreeMap,fs,io::Write,path::{Path,PathBuf},sync::mpsc};
use serde::{Deserialize,Serialize};
#[cfg(windows)]
mod logical;
#[cfg(windows)]
mod general_recording;
#[cfg(windows)]
mod model;
#[cfg(windows)]
mod code;
#[cfg(windows)]
mod nominal;

type NativeReply = Result<String, (String, Option<crate::preprocess::ExecutionDiagnostic>)>;

#[derive(Debug,Clone)]
pub(super) struct NativeSourceController {
    sender:mpsc::SyncSender<(String,mpsc::Sender<NativeReply>,Option<std::time::Instant>)>,
    #[cfg(test)] worker:std::sync::Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
}
impl NativeSourceController {
    pub(super) fn start()->Self{
        let (sender,receiver)=mpsc::sync_channel::<(String,mpsc::Sender<NativeReply>,Option<std::time::Instant>)>(2);
        // Runtime owners may contain Rc. Construct and use them only here,
        // never move a VM/permit through the HTTP worker threads.
        let builder=std::thread::Builder::new();
        #[cfg(windows)]
        let builder=builder.stack_size(32*1024*1024);
        let worker=builder.spawn(move||{
            #[cfg(windows)] let mut retained=BTreeMap::new();
            #[cfg(windows)] let mut finishes=BTreeMap::new();
            for (request,reply,submitted) in receiver {
                let rebuild_scope=crate::ddn_runtime::SavedProgramRebuildScope::begin();
                let observation=crate::storage_observation::Request::begin(submitted);
                #[cfg(windows)] let source_readback_scope=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::source_readback_scope();
                #[cfg(windows)] let result={
                    let mut diagnostic=None;
                    dispatch(&request,&mut retained,&mut finishes,&mut diagnostic)
                        .map_err(|message|(message,diagnostic))
                };
                #[cfg(not(windows))] let result={let _=request;Err(("E_SOURCE_FIRST_USE_UNSUPPORTED: Native 보호 실행을 지원하지 않습니다".into(),None))};
                #[cfg(windows)] drop(source_readback_scope);
                drop(observation);
                drop(rebuild_scope);
                let _=reply.send(result);
            }
        }).expect("Native source owner thread");
        #[cfg(not(test))] let _=worker;
        Self{sender,#[cfg(test)] worker:std::sync::Arc::new(std::sync::Mutex::new(Some(worker)))}
    }
    #[cfg(test)]
    fn close_for_cold_test(self){
        let Self{sender,worker}=self;
        drop(sender);
        // The actual owner must release its OS lease before the cold reader.
        // No sleep, replacement grant, or lock-file removal substitutes for exit.
        worker.lock().unwrap().take().unwrap().join().unwrap();
    }
    pub(super) fn call(&self,request:String)->Result<String,String>{
        self.call_with_diagnostic(request).map_err(|(message,_)|message)
    }
    pub(super) fn call_with_diagnostic(&self,request:String)->NativeReply{
        let (sender,receiver)=mpsc::channel();
        self.sender.try_send((request,sender,crate::storage_observation::submitted()))
            .map_err(|_|("E_NATIVE_SOURCE_BUSY_OR_CLOSED".into(),None))?;
        receiver.recv().map_err(|_|("E_NATIVE_SOURCE_SESSION_CLOSED".into(),None))?
    }
}
#[derive(Clone,Copy,Deserialize)]
#[serde(rename_all="snake_case")]
enum EditionSelection { Previous, Current }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditionProposal {
    project_hex:String,
    expected_revision_hex:String,
    edition:String,
    surfaces:Vec<crate::authored_project::PublicCallableSurfaceSpec>,
    ddl_source:crate::authored_project::FirstCallableDdlSource,
    consumer_source_utf8:String,
}

#[derive(Deserialize)]
#[serde(tag="operation",deny_unknown_fields)]
enum Request {
    #[cfg(test)]
    #[serde(rename="_test_finish_ready_set_failure_once")] TestFinishReadySetFailureOnce,
    #[serde(rename="general_recording_content")] GeneralRecordingContent {authored_project_json_utf8:String},
    #[serde(rename="logical_new_run")] LogicalNewRun {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest,authoring_selection:crate::runtime_surface::NativeAuthoringSelection},
    #[serde(rename="logical_save")] LogicalSave {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest,state_hash:String,
        #[serde(default)] expected_result_identity:Option<String>},
    #[serde(rename="logical_load")] LogicalLoad {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest},
    #[serde(rename="logical_verify_receipt")] LogicalVerifyReceipt {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest},
    #[serde(rename="logical_finish_begin")] LogicalFinishBegin {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest},
    #[serde(rename="logical_finish_recover")] LogicalFinishRecover {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest},
    #[serde(rename="logical_finish_commit")] LogicalFinishCommit {session_handle:String,pending_json_utf8:String},
    #[serde(rename="logical_finish_prepare")] LogicalFinishPrepare {session_handle:String,pending_json_utf8:String},
    #[serde(rename="logical_recording_result_identity")] LogicalRecordingResultIdentity {session_handle:String},
    #[serde(rename="logical_finish_notice")] LogicalFinishNotice {session_handle:String},
    #[serde(rename="logical_finish_verify")] LogicalFinishVerify {session_handle:String},
    #[serde(rename="logical_finish_close")] LogicalFinishClose {session_handle:String},
    #[serde(rename="logical_recording_stop")] LogicalRecordingStop {session_handle:String},
    #[serde(rename="logical_recording_failed_input")] LogicalRecordingFailedInput {session_handle:String,
        input_request:crate::logical_input_tape::LogicalRecordRequest,failed_input:crate::logical_input_tape::LogicalSnapshot,
        expected_prefix_result_identity:String},
    #[serde(rename="logical_new_record")] LogicalNewRecord {authored_project_json_utf8:String,input_request:crate::logical_input_tape::LogicalRecordRequest,authoring_selection:crate::runtime_surface::NativeAuthoringSelection},
    #[serde(rename="save_public_callable_source")]
    SavePublicCallableSource { source_artifact_utf8: String },
    #[serde(rename="list_public_callable_sources")]
    ListPublicCallableSources,
    #[serde(rename="load_public_callable_source")]
    LoadPublicCallableSource { project_hex: String },
    #[serde(rename="inspect_public_callable_original")]
    InspectPublicCallableOriginal {create_request_utf8:String,original_source_path:String},
    #[serde(rename="prepare_public_callable_original")]
    PreparePublicCallableOriginal {original_project_json_utf8:String,original_source_path:String,
        ddl_source:crate::authored_project::FirstCallableDdlSource,
        surfaces:Vec<crate::authored_project::PublicCallableSurfaceSpec>},
    #[serde(rename="prepare_public_callable_consumer")]
    PreparePublicCallableConsumer {original_record_json_utf8:String,create_request_utf8:String,
        consumer_source_path:String},
    #[serde(rename="read_public_callable_edition")]
    ReadPublicCallableEdition {project_hex:String, selection:EditionSelection},
    #[serde(rename="review_public_callable_edition")]
    ReviewPublicCallableEdition {candidate:EditionProposal},
    #[serde(rename="update_public_callable_edition")]
    UpdatePublicCallableEdition {candidate:EditionProposal,review_hash:String},
    #[serde(rename="run_public_callable_edition")]
    RunPublicCallableEdition {project_hex:String, selection:EditionSelection, expected_revision_hex:String},
    #[serde(rename="save_source")] Save {authored_project_json_utf8:String,
        #[serde(default)] model_source_selection:Option<crate::unit_wire::project_runtime_attachment::model_source::Anchor>,
        #[serde(default)] nominal_source_selection:Option<crate::unit_wire::project_runtime_attachment::nominal_source::Anchor>,
        #[serde(default)] code_source_selection:Option<crate::unit_wire::project_runtime_attachment::code_source::Anchor>,
        #[serde(default)] previous_authored_project_json_utf8:Option<String>,#[serde(default)] authoring_selection:Option<crate::runtime_surface::NativeAuthoringSelection>},
    #[serde(rename="new_run")] NewRun {authored_project_json_utf8:String,authoring_selection:crate::runtime_surface::NativeAuthoringSelection,seed:String},
    #[serde(rename="run")] Run {authored_project_json_utf8:String,ticks:u32},
    #[serde(rename="load")] Load {authored_project_json_utf8:String,#[serde(default)] nominal_source_selection:Option<crate::unit_wire::project_runtime_attachment::nominal_source::Anchor>},
    #[serde(rename="connect")] Connect {authored_project_json_utf8:String,#[serde(default)] restore_only:bool},
    #[serde(rename="run_to")] RunTo {authored_project_json_utf8:String,next_tick:String},
    #[serde(rename="save_world")] SaveWorld {authored_project_json_utf8:String,next_tick:String,state_hash:String},
}
#[derive(Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
struct SourceLocator {
    project_hex:String,source_revision_hex:String,source_integrity_hex:String,grant_locator:String,
    authored_project_json_utf8:String,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldLocator {parent_hex:String,revision_hex:String,saved_artifact_json_utf8:String}

fn read<T:serde::de::DeserializeOwned>(path:&Path)->Result<T,String>{
    if fs::metadata(path).map_err(|_|"E_NATIVE_SOURCE_LOCATOR_READ")?.len()>8*1024*1024{return Err("E_NATIVE_SOURCE_LOCATOR_SIZE".into());}
    let bytes=fs::read(path).map_err(|e|format!("E_NATIVE_SOURCE_LOCATOR_READ: {e}"))?;
    if bytes.len()>8*1024*1024{return Err("E_NATIVE_SOURCE_LOCATOR_SIZE".into());}
    serde_json::from_slice(&bytes).map_err(|_|"E_NATIVE_SOURCE_LOCATOR_FIELDS".into())
}
fn write_once<T:Serialize>(path:&Path,value:&T)->Result<(),String>{
    // Never overwrite a previous locator or claim a partially written one is
    // complete. A crash gap remains an explicit recovery error, not repair.
    let bytes=serde_json::to_vec(value).map_err(|_|"E_NATIVE_SOURCE_LOCATOR_ENCODING")?;
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e|format!("E_NATIVE_SOURCE_LOCATOR_CREATE: {e}"))?;
    file.write_all(&bytes).and_then(|_|file.sync_all())
        .map_err(|e|format!("E_NATIVE_SOURCE_LOCATOR_UNCONFIRMED: {e}"))
}
fn retain_source_cache(root:&Path,source:&SourceLocator)->Result<(),String>{
    // Non-authoritative immutable cache only, produced after actual source
    // admission. Filenames never admit a source, root, Run or world.
    identity(&source.source_integrity_hex)?;
    let path=root.join(format!("source-{}.json",source.source_integrity_hex));
    if path.exists(){if read::<SourceLocator>(&path)?!=*source{return Err("E_NATIVE_SOURCE_CACHE_READBACK".into());}Ok(())}
    else{write_once(&path,source)}
}
fn identity(text:&str)->Result<Vec<u8>,String>{
    if text.is_empty()||text.len()>8192||text.len()%2!=0||!text.bytes().all(|v|v.is_ascii_digit()||(b'a'..=b'f').contains(&v)) {
        return Err("E_NATIVE_SOURCE_LOCATOR_IDENTITY".into());
    }hex::decode(text).map_err(|_|"E_NATIVE_SOURCE_LOCATOR_IDENTITY".into())
}
fn revision()->Result<Vec<u8>,String>{let mut bytes=[0;16];getrandom::fill(&mut bytes).map_err(|_|"E_NATIVE_SOURCE_RANDOM")?;Ok(bytes.to_vec())}
fn local_source_base()->Result<PathBuf,String>{
    let base=std::env::var_os("LOCALAPPDATA").ok_or("E_NATIVE_SOURCE_LOCAL_ROOT")?;
    Ok(PathBuf::from(base).join("Ddonilang").join("WorkbenchSource"))
}


fn root(project:&str)->Result<PathBuf,String>{
    if project.len()!=64{return Err("E_NATIVE_SOURCE_PROJECT_IDENTITY".into());}identity(project)?;
    let base=local_source_base()?;
    fs::create_dir_all(&base).map_err(|e|format!("E_NATIVE_SOURCE_LOCAL_ROOT: {e}"))?;
    Ok(base.join(project))
}
#[cfg(windows)]
fn selected_head(path:&Path,project:&str,locator:&str,authored:&str,retained:&mut BTreeMap<String,crate::runtime_surface::NativeFirstSourceWorldRetained>)
    ->Result<crate::runtime_surface::NativeAuthoringSelection,String>{
    if let Some(owner)=retained.get_mut(project){owner.authoring_selection(&identity(project)?,locator,authored)}
    else{crate::runtime_surface::native_authoring_selection(path,&identity(project)?,locator,authored)}
}

#[cfg(windows)]
fn dispatch(text:&str,retained:&mut BTreeMap<String,crate::runtime_surface::NativeFirstSourceWorldRetained>,
    finishes:&mut BTreeMap<String,crate::runtime_surface::NativeLogicalFinishSessionV1>,
    diagnostic_out:&mut Option<crate::preprocess::ExecutionDiagnostic>)->Result<String,String>{
    use crate::runtime_surface as runtime;
    let request:Request=serde_json::from_str(text).map_err(|_|"E_NATIVE_SOURCE_REQUEST_FIELDS")?;
    if let Some(authored)=request_authored(&request){
        // Load verifies custody, not today's source admission. It neither runs
        // nor saves. The real v4 store, grant and head are checked by dispatch.
        let nominal_selected=matches!(&request,Request::Load{nominal_source_selection:Some(_),..});
        let nominal_candidate=if nominal_selected {true}
            else if matches!(&request,Request::Save{..}) {
                crate::unit_wire::project_runtime_attachment::nominal_source::checked_project(authored)?.is_some()
            }else{false};
        if nominal_candidate {
            if matches!(&request,Request::Save{model_source_selection:Some(_),..}|Request::Save{code_source_selection:Some(_),..}){return Err("E_NOMINAL_SOURCE_SELECTION_KIND".into());}
            return nominal::dispatch(&request,authored);
        }
        if matches!(&request,Request::Save{nominal_source_selection:Some(_),..}){return Err("E_NOMINAL_SOURCE_REQUIRED".into());}
        let code=if matches!(&request,Request::Load{..}){
            crate::unit_wire::project_runtime_attachment::code_source::custody_project(authored)?
        }else{crate::unit_wire::project_runtime_attachment::code_source::checked_project(authored)?};
        if code.is_some(){
            if matches!(&request,Request::Save{model_source_selection:Some(_),..}){return Err("E_CODE_SOURCE_SELECTION_KIND".into());}
            return code::dispatch(&request,authored);
        }
        if matches!(&request,Request::Save{code_source_selection:Some(_),..}){return Err("E_CODE_SOURCE_REQUIRED".into());}
        if crate::unit_wire::project_runtime_attachment::model_source::checked_project(authored)?.is_some(){
            return model::dispatch(&request,authored);
        }
    }
    if let Some(authored)=request_authored(&request){
        // A structural selector only; open and the checked B1 owner below
        // reject malformed, stale, unregistered or ambiguous input.
        let j1=serde_json::from_str::<serde_json::Value>(authored).ok()
            .and_then(|value|value.get("source_profile").and_then(serde_json::Value::as_str).map(str::to_owned))
            .as_deref()==Some(ddonirang_lang::ddl::J1_SOURCE_PROFILE);
        if j1{
            let project=crate::authored_project::open(authored.as_bytes())?;
            if crate::authored_project::selected_logic_dependency(&project)?.is_some(){
                return dispatch_logic_source(&request,authored,&project);
            }
        }
    }
    let (authored,ticks,is_save)=match &request {
        Request::SavePublicCallableSource{source_artifact_utf8}=>return save_public_callable_source(source_artifact_utf8),
        Request::ListPublicCallableSources=>return list_public_callable_sources(),
        Request::LoadPublicCallableSource{project_hex}=>return load_public_callable_source(project_hex),
        Request::InspectPublicCallableOriginal{create_request_utf8,original_source_path}=>{
            return crate::authored_project::inspect_public_callable_original(create_request_utf8,original_source_path,diagnostic_out);
        }
        Request::PreparePublicCallableOriginal{original_project_json_utf8,original_source_path,ddl_source,surfaces}=>{
            let input=serde_json::json!({"original_project_json_utf8":original_project_json_utf8,
                "original_source_path":original_source_path,"ddl_source":ddl_source,"surfaces":surfaces,
                "revision_hex":hex::encode(revision()?)});
            let record=crate::authored_project::prepare_public_callable_original_diagnostic(
                &input.to_string(),Some(diagnostic_out))?;
            return Ok(serde_json::json!({"kind":"public_callable_original_prepared",
                "original_record_json_utf8":record}).to_string());
        }
        Request::PreparePublicCallableConsumer{original_record_json_utf8,create_request_utf8,consumer_source_path}=>{
            return prepare_workbench_consumer(original_record_json_utf8,create_request_utf8,consumer_source_path);
        }
        Request::ReadPublicCallableEdition{project_hex,selection}=>{
            let (_,source,updated)=selected_public_edition(project_hex,*selection)?;
            return public_edition_view(&source,updated);
        }
        Request::ReviewPublicCallableEdition{candidate}|Request::UpdatePublicCallableEdition{candidate,..}=>{
            let (mut adapter,source,updated)=selected_public_edition(&candidate.project_hex,EditionSelection::Current)?;
            if updated || identity(&candidate.expected_revision_hex)?!=source.revision(){
                return Err("E_DDL_EDITION_PARENT_MISMATCH".into());
            }
            let diagnostic=matches!(&request,Request::ReviewPublicCallableEdition{..}).then_some(diagnostic_out);
            let checked=crate::authored_project::check_public_edition_from_current(&adapter,&source,
                &candidate.edition,&candidate.surfaces,candidate.ddl_source.clone(),candidate.consumer_source_utf8.clone(),diagnostic)?;
            let digest=checked.review_hash()?;
            if let Request::UpdatePublicCallableEdition{review_hash,..}=&request{
                if review_hash!=&digest{return Err("E_DDL_EDITION_REVIEW_CHANGED".into());}
                let saved=checked.prepare(&revision()?)?.save(&mut adapter)?;
                return public_edition_view(&saved,true);
            }
            return Ok(serde_json::json!({
                "kind":"public_callable_edition_review","project_hex":candidate.project_hex,
                "revision_hex":hex::encode(source.revision()),"review_hash":digest,
            }).to_string());
        }
        Request::RunPublicCallableEdition{project_hex,selection,expected_revision_hex}=>{
            let (_adapter,source,_)=selected_public_edition(project_hex,*selection)?;
            if identity(expected_revision_hex)?!=source.revision(){return Err("E_DDL_EDITION_REVISION_SELECTION".into());}
            return crate::authored_project::run_public_source_diagnostic(&source,Some(diagnostic_out));
        }
        #[cfg(test)]
        Request::TestFinishReadySetFailureOnce=>{
            crate::snapshot_v2_durable_transaction::native::test_arm_finish_ready_set_failure();
            return Ok(serde_json::json!({"armed":true}).to_string());
        },
        Request::GeneralRecordingContent{authored_project_json_utf8}=>{
            return general_recording::content(authored_project_json_utf8,retained);
        },
        Request::LogicalNewRun{authored_project_json_utf8,input_request,authoring_selection}=>{
            return logical::new_run(authored_project_json_utf8,input_request,authoring_selection,retained);
        },
        Request::LogicalSave{authored_project_json_utf8,input_request,state_hash,expected_result_identity}=>{
            if let Some(value)=expected_result_identity{
                let digest=value.strip_prefix("sha256:").ok_or("E_NATIVE_SOURCE_RESULT_IDENTITY_FORMAT")?;
                if digest.len()!=64||!digest.bytes().all(|v|v.is_ascii_digit()||(b'a'..=b'f').contains(&v)){return Err("E_NATIVE_SOURCE_RESULT_IDENTITY_FORMAT".into());}
            }
            return logical::dispatch(authored_project_json_utf8,input_request,Some(state_hash),expected_result_identity.as_deref(),retained);
        },
        Request::LogicalLoad{authored_project_json_utf8,input_request}=>{
            return logical::dispatch(authored_project_json_utf8,input_request,None,None,retained);
        },
        Request::LogicalVerifyReceipt{authored_project_json_utf8,input_request}=>{
            return logical::verify_receipt(authored_project_json_utf8,input_request,retained);
        },
        Request::LogicalNewRecord{authored_project_json_utf8,input_request,authoring_selection}=>{
            return logical::new_record(authored_project_json_utf8,input_request,retained,authoring_selection);
        },
        Request::LogicalFinishBegin{authored_project_json_utf8,input_request}=>{
            if finishes.len()>=16{return Err("E_NATIVE_FINISH_SESSION_LIMIT".into());}
            let mut owner=logical::begin_finish(authored_project_json_utf8,input_request,retained)?;
            let registration=owner.register_frontend()?;
            let handle=hex::encode(revision()?);
            if finishes.contains_key(&handle){return Err("E_NATIVE_FINISH_HANDLE_COLLISION".into());}
            finishes.insert(handle.clone(),owner);
            return Ok(serde_json::json!({"session_handle":handle,"registration":registration}).to_string());
        },
        Request::LogicalFinishRecover{authored_project_json_utf8,input_request}=>{
            return logical::recover_finish(authored_project_json_utf8,input_request,retained);
        },
        Request::LogicalFinishCommit{session_handle,pending_json_utf8}=>{
            return finishes.get_mut(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .finish(pending_json_utf8).map(|value|value.to_string());
        },
        Request::LogicalFinishPrepare{session_handle,pending_json_utf8}=>{
            return finishes.get_mut(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .preview(pending_json_utf8).map(|value|value.to_string());
        },
        Request::LogicalRecordingResultIdentity{session_handle}=>{
            return finishes.get(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .result_identity().map(|value|value.to_string());
        },
        Request::LogicalFinishNotice{session_handle}=>{
            return finishes.get(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .notice().map(|value|value.to_string());
        },
        Request::LogicalFinishVerify{session_handle}=>{
            return finishes.get(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .verify_receipt().map(|value|value.to_string());
        },
        Request::LogicalFinishClose{session_handle}=>{
            finishes.remove(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?;
            return Ok(serde_json::json!({"closed":true}).to_string());
        },
        Request::LogicalRecordingStop{session_handle}=>{
            return finishes.get_mut(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .stop().map(|value|value.to_string());
        },
        Request::LogicalRecordingFailedInput{session_handle,input_request,failed_input,expected_prefix_result_identity}=>{
            return finishes.get_mut(session_handle).ok_or("E_NATIVE_FINISH_SESSION_CLOSED")?
                .fail_at_input(input_request,failed_input,expected_prefix_result_identity).map(|value|value.to_string());
        },
        Request::Save{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,true),
        Request::NewRun{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,false),
        Request::Run{authored_project_json_utf8,ticks}=>(authored_project_json_utf8,*ticks,false),
        Request::Load{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,false),
        Request::Connect{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,false),
        Request::RunTo{authored_project_json_utf8,..}|Request::SaveWorld{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,false),
    };
    if matches!(&request,Request::Save{model_source_selection:Some(_),..}){return Err("E_MODEL_SOURCE_REQUIRED".into());}
    if matches!(request,Request::Run{..})&&(ticks==0||ticks>2000){return Err("E_NATIVE_SOURCE_MADI_LIMIT: 한 번에 1~2000마디를 지정하세요".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str().to_owned();
    let path=root(&project)?;let locator_path=path.join("source.locator.json");
    if !locator_path.exists(){
        if !is_save{return Err("E_NATIVE_SOURCE_SAVE_REQUIRED: 먼저 현재 원문을 저장하세요".into());}
        // Explicit save may re-confirm the exact already committed initial
        // source. Existing/unknown/Started stores are never re-provisioned.
        let source=if path.exists(){
            runtime::reconfirm_initial_authored_source_native(&path,&identity(&project)?,authored)?
        }else{
            fs::create_dir(&path).map_err(|e|format!("E_NATIVE_SOURCE_STORE_EXISTS_OR_UNAVAILABLE: {e}"))?;
            runtime::save_initial_authored_source_native(authored,&revision()?,&path)?
        };
        let locator=SourceLocator{project_hex:hex::encode(source.project_identity),source_revision_hex:hex::encode(source.revision_identity),
            source_integrity_hex:hex::encode(source.revision_integrity),grant_locator:source.grant_locator.ok_or("E_NATIVE_SOURCE_GRANT_MISSING")?,
            authored_project_json_utf8:authored.clone()};
        write_once(&locator_path,&locator)?;
    }
    let mut source:SourceLocator=read(&locator_path)?;
    if source.project_hex!=project {
        return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 저장된 원문과 다릅니다. 기존 저장본은 보존했습니다".into());
    }
    // Recovery reads carry no execution permit. The Native owner proves the
    // entire closed pre-world state; the caller flag below only selects the
    // explicit retry route, which repeats that proof before protected CAS.
    let retry=matches!(&request,Request::NewRun{authoring_selection,..} if authoring_selection.kind==2);
    if matches!(&request,Request::Load{..})||retry {
        match runtime::read_failed_first_source_native(&path,&identity(&project)?,&source.grant_locator) {
            Ok((actual,selection,recorded))=>{
                let payload:serde_json::Value=serde_json::from_str(&actual.project_payload_utf8).map_err(|_|"E_SOURCE_RETRY_PAYLOAD")?;
                let current=payload["authored_project_json_utf8"].as_str().ok_or("E_SOURCE_RETRY_PAYLOAD")?;
                if !retry {
                    return Ok(serde_json::json!({"kind":"native_source_current","authored_project_json_utf8":current,
                        "source_artifact_utf8":recorded,"requires_new_run":true,"first_failure_recovery":true,
                        "authoring_selection":selection}).to_string());
                }
                let Request::NewRun{authoring_selection,seed,..}=&request else{unreachable!()};
                if current!=authored{return Err("E_SOURCE_RETRY_SOURCE_CHANGED".into());}
                if retained.len()>=16||retained.contains_key(&project){return Err("E_NATIVE_SOURCE_SESSION_LIMIT".into());}
                let seed_value:u64=seed.parse().map_err(|_|"E_NATIVE_SOURCE_SEED")?;
                if seed_value.to_string()!=*seed{return Err("E_NATIVE_SOURCE_SEED".into());}
                let session=runtime::begin_retry_first_source_native(&path,&identity(&project)?,&source.grant_locator,authored,authoring_selection)?;
                source.source_revision_hex=hex::encode(&actual.revision_identity);source.source_integrity_hex=hex::encode(actual.revision_integrity);
                return commit_first_source_session(&path,source,project,authored,&request,1,seed_value,session,retained);
            },
            // Existing ordinary/committed readers retain their own full gates.
            // A retry request never falls through to one of those paths.
            Err(error) if retry=>return Err(error),
            Err(_)=>{},
        }
    }
    // Explicit changed-source requests carry their pre-edit selected head.
    // Never infer that selection from whichever world happens to be current.
    if let Request::Save{previous_authored_project_json_utf8:Some(previous),authoring_selection:Some(selection),..}=&request {
        if previous!=authored {
            let next=revision()?;
            let actual=if let Some(owner)=retained.get_mut(&project){owner.save_changed_source(authored,previous,&next,&source.grant_locator,selection)?}
                else{runtime::save_changed_authored_source_native(authored,previous,&next,&path,&source.grant_locator,selection)?};
            retained.remove(&project);
            let selected=runtime::native_authoring_selection(&path,&identity(&project)?,&source.grant_locator,authored)?;
            let saved=SourceLocator{project_hex:project.clone(),source_revision_hex:hex::encode(&actual.revision_identity),source_integrity_hex:hex::encode(actual.revision_integrity),
                grant_locator:source.grant_locator.clone(),authored_project_json_utf8:authored.clone()};
            retain_source_cache(&path,&saved)?;
            return Ok(serde_json::json!({"kind":"native_source_saved","authored_project_json_utf8":authored,"requires_new_run":true,"authoring_selection":selected}).to_string());
        }
    }
    let ready=if let Some(owner)=retained.get_mut(&project){owner.current_authoring_source(&identity(&project)?,&source.grant_locator)?}
        else{runtime::current_authoring_source_native(&path,&identity(&project)?,&source.grant_locator)?};
    if let Some((actual,selection))=ready {
        let payload:serde_json::Value=serde_json::from_str(&actual.project_payload_utf8).map_err(|_|"E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
        let current=payload["authored_project_json_utf8"].as_str().ok_or("E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
        if matches!(request,Request::Load{..})||is_save {
            if is_save&&current!=authored{return Err("E_NATIVE_SOURCE_AUTHORING_SELECTION_REQUIRED: 원문을 열 때 받은 기준 판으로 수정 저장하세요".into());}
            let saved=SourceLocator{project_hex:project.clone(),source_revision_hex:hex::encode(&actual.revision_identity),source_integrity_hex:hex::encode(actual.revision_integrity),
                grant_locator:source.grant_locator.clone(),authored_project_json_utf8:current.into()};
            retain_source_cache(&path,&saved)?;retained.remove(&project);
            return Ok(serde_json::json!({"kind":if is_save{"native_source_saved"}else{"native_source_current"},"authored_project_json_utf8":current,
                "requires_new_run":true,"authoring_selection":selection}).to_string());
        }
        if let Request::NewRun{authoring_selection,seed,..}=&request {
            if current!=authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 현재 저장 원문을 다시 여세요".into());}
            if retained.len()>=16&&!retained.contains_key(&project){return Err("E_NATIVE_SOURCE_SESSION_LIMIT".into());}
            let seed_value:u64=seed.parse().map_err(|_|"E_NATIVE_SOURCE_SEED")?;
            if seed_value.to_string()!=*seed{return Err("E_NATIVE_SOURCE_SEED".into());}
            retained.remove(&project);
            let next=revision()?;
            let prepared=runtime::begin_new_authored_source_native(&path,&identity(&project)?,&source.grant_locator,authored,authoring_selection)?
                .execute(1,seed_value,None,None)?.prepare(&next)?;
            let parent=hex::encode(&actual.revision_identity);
            prepared.stage_restore_cache(|artifact,digest|write_once(&path.join(format!("world-{}.json",hex::encode(digest))),
                &WorldLocator{parent_hex:parent.clone(),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()}))?;
            let mut run=prepared.write_prepared()?.commit()?;
            let artifact=run.saved_runtime_artifact()?.to_owned();
            let snapshot=run.browser_snapshot(&actual.revision_identity,&next,&source.grant_locator)?;
            let selected=run.authoring_selection(&identity(&project)?,&source.grant_locator,authored)?;
            let receipt=crate::unit_wire::SnapshotJsonOwner::clone_from_value(run.summary()?["input_receipts"].as_array().and_then(|r|r.last()).ok_or("E_NATIVE_SOURCE_INPUT_RECEIPT")?);
            retained.insert(project,run);
            return Ok(serde_json::json!({"saved_artifact_json_utf8":artifact,"snapshot_hex":hex::encode(snapshot),"parent_hex":parent,"revision_hex":hex::encode(next),
                "first_frame_pending":true,"first_input_receipt":receipt.into_parent_value(),"authoring_selection":selected}).to_string());
        }
        return Err("E_SOURCE_NEW_RUN_REQUIRED: 수정 원문은 저장됐습니다. 명시적으로 새 실행을 시작하세요. 이전 세계를 이어 붙이지 않았습니다".into());
    }
    if matches!(request,Request::NewRun{..}){return Err("E_SOURCE_NEW_RUN_NOT_READY: 새 실행 가능한 수정 원문 판이 아닙니다".into());}
    let world_path=path.join("world.locator.json");
    let restore_only=matches!(&request,Request::Connect{restore_only:true,..});
    if world_path.exists(){
        let digest=if let Some(owner)=retained.get_mut(&project){owner.current_digest(&identity(&project)?,&source.grant_locator)?}
            else{runtime::native_source_current_digest(&path,&identity(&project)?,&source.grant_locator)?};
        let world:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
        source.source_revision_hex=world.parent_hex.clone();
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(world.saved_artifact_json_utf8.as_bytes())?;
        let saved_owner=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
        let current=saved_owner.project().save()?;
        if current!=*authored {
            if !matches!(request,Request::Load{..}) {
                return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 저장된 세계의 원문과 다릅니다".into());
            }
            // A stale browser locator may select this project's current source,
            // but cannot select a historical world or authorize an execution.
            // Readmit the actual protected current world before returning bytes.
            if let Some(owner)=retained.get_mut(&project) {
                owner.browser_snapshot(&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&source.grant_locator)?;
            } else {
                runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,
                    &identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&path,&source.grant_locator)?;
            }
            let selected=selected_head(&path,&project,&source.grant_locator,&current,retained)?;
            return Ok(serde_json::json!({"kind":"native_source_current",
                "authored_project_json_utf8":current,"requires_new_run":false,"authoring_selection":selected}).to_string());
        }
        if (is_save||matches!(request,Request::Load{..}))&&logical::is_logical_artifact(&world.saved_artifact_json_utf8)? {
            // This is source reopening only, not logical execution admission.
            runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,&identity(&world.parent_hex)?,
                &identity(&world.revision_hex)?,&path,&source.grant_locator)?;
            let selected=selected_head(&path,&project,&source.grant_locator,authored,retained)?;
            return Ok(serde_json::json!({"kind":if is_save{"native_source_saved"}else{"native_logical_saved_source"},"authored_project_json_utf8":authored,"authoring_selection":selected}).to_string());
        }
        if restore_only&&logical::is_logical_artifact(&world.saved_artifact_json_utf8)? {
            return Err("E_NATIVE_SOURCE_LOGICAL_CHECKPOINT_REQUIRED: 논리 입력 실행 상태는 논리 입력 복원 경로로 여세요".into());
        }
        if is_save||matches!(request,Request::Connect{..}) {
            let parent=identity(&source.source_revision_hex)?;let revision=identity(&world.revision_hex)?;
            let snapshot=if let Some(owner)=retained.get_mut(&project){owner.browser_snapshot(&parent,&revision,&source.grant_locator)?}
                else{runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,&parent,&revision,&path,&source.grant_locator)?};
            // Re-saving exactly the committed source is a checked no-op, not
            // permission to edit a consumed source epoch or publish a world.
            let selected=selected_head(&path,&project,&source.grant_locator,authored,retained)?;
            if is_save{return Ok(serde_json::json!({"kind":"native_source_saved","authored_project_json_utf8":authored,"authoring_selection":selected}).to_string());}
            return Ok(serde_json::json!({"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
                "snapshot_hex":hex::encode(snapshot),"parent_hex":source.source_revision_hex,"revision_hex":world.revision_hex,"first_frame_pending":false,"authoring_selection":selected}).to_string());
        }
        let ticks=match &request{
            Request::RunTo{next_tick,..}|Request::SaveWorld{next_tick,..}=>{
                let next:u64=next_tick.parse().map_err(|_|"E_NATIVE_SOURCE_MADI_IDENTITY")?;
                if next.to_string()!=*next_tick{return Err("E_NATIVE_SOURCE_MADI_IDENTITY".into());}
                let snapshot=if let Some(owner)=retained.get_mut(&project){owner.browser_snapshot(&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&source.grant_locator)?}
                    else{runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&path,&source.grant_locator)?};
                let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(world.saved_artifact_json_utf8.as_bytes())?;
                let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
                let pesi=crate::project_snapshot_identity::current::issue_saved_project(&owned)?;
                let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,pesi.pesi().as_raw()).map_err(|_|"E_NATIVE_SOURCE_FRAME")?;
                let suffix=next.checked_sub(layout.world.next_tick).and_then(|n|u32::try_from(n).ok()).filter(|n|*n<=2000).ok_or("E_NATIVE_SOURCE_MADI_LIMIT")?;
                suffix
            },_=>ticks};
        let result=if let Some(owner)=retained.get_mut(&project) {
            owner.resume_checkpoint(&identity(&source.source_revision_hex)?,&identity(&world.revision_hex)?,&source.grant_locator,ticks)?
        }else{runtime::resume_first_saved_source_world_native(&world.saved_artifact_json_utf8,
            &identity(&source.source_revision_hex)?,&identity(&world.revision_hex)?,&path,&source.grant_locator,ticks)?};
        if let Request::SaveWorld{state_hash,..}=&request{
            if result["state_hash"].as_str()!=Some(state_hash){return Err("E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH: 화면과 Native 세계가 다릅니다. 기존 저장본은 보존했습니다".into());}
            if ticks>0 {
                let next=revision()?;
                let stage=|artifact:&str,digest:&[u8;32]|write_once(&path.join(format!("world-{}.json",hex::encode(digest))),
                    &WorldLocator{parent_hex:world.revision_hex.clone(),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()});
                let saved=if let Some(owner)=retained.get_mut(&project){owner.save_continuation(&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&next,&source.grant_locator,ticks,stage)?}
                    else{runtime::save_continued_source_world_native(&world.saved_artifact_json_utf8,&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&next,&path,&source.grant_locator,ticks,stage)?};
                let selected=selected_head(&path,&project,&source.grant_locator,authored,retained)?;
                return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_world_saved","summary":saved,"authoring_selection":selected})).compact_text());
            }
            let selected=selected_head(&path,&project,&source.grant_locator,authored,retained)?;
            return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_world_saved","summary":result,"authoring_selection":selected})).compact_text());
        }
        // Cold result is not a new durable checkpoint. Keep its exact scope.
        let selected=selected_head(&path,&project,&source.grant_locator,authored,retained)?;
        return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_cold_result",
            "authored_project_json_utf8":authored,"summary":result,"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
            "continued_state_saved":false,"authoring_selection":selected})).compact_text());
    }
    // The restore button must never consume the first-run grant, even if
    // there was no checkpoint or the current source has no saved world.
    if restore_only{return Err("E_NATIVE_SOURCE_WORLD_REQUIRED: 저장된 실행 상태가 없습니다".into());}
    if matches!(request,Request::RunTo{..}|Request::SaveWorld{..}){return Err("E_NATIVE_SOURCE_WORLD_REQUIRED: 먼저 저장한 원문을 실행하세요".into());}
    // The protected Native source head is authoritative, never this physical
    // locator's original revision. Source editing uses the existing S-chain.
    let mut actual=runtime::read_authored_source_native(&path,&identity(&source.project_hex)?,&source.grant_locator)?;
    let current:serde_json::Value=serde_json::from_str(&actual.project_payload_utf8).map_err(|_|"E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
    let current_text=current["authored_project_json_utf8"].as_str().ok_or("E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
    if current_text!=authored {
        if !is_save{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 원문을 먼저 갱신 저장하세요".into());}
        actual=runtime::update_authored_source_native(authored,&revision()?,&path,&identity(&project)?,&source.grant_locator,
            &actual.revision_identity,&actual.revision_integrity)?;
    }
    source.source_revision_hex=hex::encode(actual.revision_identity);
    source.source_integrity_hex=hex::encode(actual.revision_integrity);
    if ticks==0&&!matches!(request,Request::Connect{..}) {
        let actual=runtime::read_authored_source_native(&path,&identity(&source.project_hex)?,&source.grant_locator)?;
        if hex::encode(actual.revision_identity)!=source.source_revision_hex||hex::encode(actual.revision_integrity)!=source.source_integrity_hex {
            return Err("E_NATIVE_SOURCE_SOURCE_CURRENT".into());
        }
        return Ok(serde_json::json!({"kind":"native_source_saved","authored_project_json_utf8":authored}).to_string());
    }
    if retained.contains_key(&project){return Err("E_NATIVE_SOURCE_FIRST_RUN_ALREADY_RETAINED".into());}
    if retained.len()>=16{return Err("E_NATIVE_SOURCE_SESSION_LIMIT: 새 작품을 실행하려면 작업대를 종료한 뒤 다시 여세요".into());}
    let integrity:[u8;32]=identity(&source.source_integrity_hex)?.try_into().map_err(|_|"E_NATIVE_SOURCE_LOCATOR_IDENTITY")?;
    let session=runtime::begin_first_authored_source_native(&path,&identity(&project)?,&source.grant_locator,
        &identity(&source.source_revision_hex)?,&integrity)?;
    commit_first_source_session(&path,source,project,authored,&request,ticks,0,session,retained)
}

#[cfg(windows)]
fn commit_first_source_session(path:&Path,source:SourceLocator,project:String,authored:&str,request:&Request,ticks:u32,seed:u64,
    session:crate::runtime_surface::NativeFirstSourceWorldSession,retained:&mut BTreeMap<String,crate::runtime_surface::NativeFirstSourceWorldRetained>)->Result<String,String>{
    let world_path=path.join("world.locator.json");
    let next=revision()?;
    let prepared=session.execute(if matches!(request,Request::Connect{..}|Request::NewRun{..}){1}else{ticks},seed,None,None)?.prepare_recording(&next)?;
    prepared.stage_restore_cache(|artifact,digest|{
        let candidate=WorldLocator{parent_hex:source.source_revision_hex.clone(),revision_hex:hex::encode(&next),saved_artifact_json_utf8:artifact.into()};
        write_once(&path.join(format!("world-{}.json",hex::encode(digest))),&candidate)?;
        write_once(&world_path,&candidate)
    })?;
    let run=prepared.commit()?;
    let world=WorldLocator{parent_hex:source.source_revision_hex.clone(),revision_hex:hex::encode(next),saved_artifact_json_utf8:run.saved_runtime_artifact()?.into()};
    let report=run.report_json()?;
    // Keep the active pinned owner: a second adapter cannot acquire its lease.
    let mut run=run;
    let digest=run.current_digest(&identity(&project)?,&source.grant_locator)?;
    let staged:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
    if staged.parent_hex!=world.parent_hex||staged.revision_hex!=world.revision_hex||staged.saved_artifact_json_utf8!=world.saved_artifact_json_utf8{
        return Err("E_NATIVE_SOURCE_CACHE_READBACK: 준비 자료가 확정된 세계와 다릅니다. 기존 자료를 보존했습니다".into());
    }
    retained.insert(project,run); // Same protected owner survives the response.
    if matches!(request,Request::Connect{..}|Request::NewRun{..}) {
        let owner=retained.get_mut(&source.project_hex).ok_or("E_NATIVE_SOURCE_SESSION_CLOSED")?;
        let snapshot=owner.browser_snapshot(&identity(&source.source_revision_hex)?,&identity(&world.revision_hex)?,&source.grant_locator)?;
        let selected=owner.authoring_selection(&identity(&source.project_hex)?,&source.grant_locator,authored)?;
        let receipt=owner.summary()?["input_receipts"].as_array().and_then(|rows|rows.last()).ok_or("E_NATIVE_SOURCE_INPUT_RECEIPT")?;
        return Ok(serde_json::json!({"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
            "snapshot_hex":hex::encode(snapshot),"parent_hex":source.source_revision_hex,"revision_hex":world.revision_hex,"first_frame_pending":true,
            "first_input_receipt":receipt,"authoring_selection":selected}).to_string());
    }
    Ok(report)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn transport_is_closed_not_a_permit_factory(){
        for raw in [r#"{"operation":"run","authored_project_json_utf8":"{}","ticks":1,"permit":true}"#,
            r#"{"operation":"save_source","authored_project_json_utf8":"{}","root":"C:/foreign"}"#] {
            assert!(serde_json::from_str::<Request>(raw).is_err());
        }
        assert!(identity("01").is_ok());for text in ["","AA","../aa","a"]{assert!(identity(text).is_err());}
    }
    #[test]
    #[cfg(windows)]
    fn actual_controller_keeps_native_owner_and_saved_source(){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let raw="병사:임자 := { 이름:글. }. 실행:움직씨 := { (매마디)마다 { 철수:병사 := (이름=\"철수\")인 병사. 표적:(병사)가리킴 := 철수 가리키기. 읽은것:(병사,가리킴안됨)맺음 := 표적 읽기. 읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 값.이름 보여주기. }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { }. }. (대상=표적) 폐기하기. 색 <- #빨강. }. }.";
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:raw.into()}]},String::new()).unwrap().save().unwrap();
        let controller=NativeSourceController::start();
        let call=|operation:&str,ticks:Option<u32>|{
            let mut request=serde_json::json!({"operation":operation,"authored_project_json_utf8":project});
            if let Some(ticks)=ticks{request["ticks"]=ticks.into();}
            controller.call(request.to_string())
        };
        let saved=call("save_source",None).unwrap();assert!(saved.contains("native_source_saved"));
        let first:serde_json::Value=serde_json::from_str(&call("run",Some(1)).unwrap()).unwrap();
        assert_eq!(first["summary"]["output_log_texts"],serde_json::json!(["철수"]));
        let artifact=first["saved_artifact_json_utf8"].as_str().unwrap();
        let loaded:serde_json::Value=serde_json::from_str(&call("load",None).unwrap()).unwrap();
        assert_eq!(loaded["summary"]["next_tick_id"],1);
        assert_eq!(loaded["saved_artifact_json_utf8"],artifact);
        let resumed:serde_json::Value=serde_json::from_str(&call("run",Some(1)).unwrap()).unwrap();
        assert_eq!(resumed["summary"]["next_tick_id"],2);
        assert_eq!(resumed["summary"]["output_display"]["texts"],serde_json::json!(["철수","철수"]));
        assert_eq!(resumed["continued_state_saved"],false);
        assert_eq!(resumed["saved_artifact_json_utf8"],artifact);
        assert!(call("run",Some(0)).unwrap_err().contains("MADI_LIMIT"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&call("load",None).unwrap()).unwrap()["saved_artifact_json_utf8"],artifact);
    }
    #[test]
    #[cfg(windows)]
    fn actual_controller_authoring_ready_explicit_run_and_second_cycle(){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let source=SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"실행:움직씨 := { (매마디)마다 { 값 <- 1. 값 보여주기. }. }.".into()}]};
        let original=ProjectArtifact::create("v1-core-v25".into(),source.clone(),String::new()).unwrap().save().unwrap();
        let controller=NativeSourceController::start();
        let call=|operation:&str,authored:&str,fields:serde_json::Value|{
            let mut request=fields;request["operation"]=operation.into();request["authored_project_json_utf8"]=authored.into();
            controller.call(request.to_string()).map(|s|serde_json::from_str::<serde_json::Value>(&s).unwrap())
        };
        call("save_source",&original,serde_json::json!({})).unwrap();
        let first=call("connect",&original,serde_json::json!({})).unwrap();
        let mut previous=original.clone();let mut selected=first["authoring_selection"].clone();
        for amount in [2,3] {
            let mut changed=source.clone();changed.files[1].text=changed.files[1].text.replace("<- 1.",&format!("<- {amount}."));
            let edited=crate::authored_project::operate("update",&serde_json::json!({"original_json":previous,"source_snapshot":changed}).to_string()).unwrap();
            let saved=call("save_source",&edited,serde_json::json!({"previous_authored_project_json_utf8":previous,"authoring_selection":selected})).unwrap();
            assert_eq!(saved["requires_new_run"],true);
            let cold=call("load",&previous,serde_json::json!({})).unwrap();
            assert_eq!(cold["kind"],"native_source_current");assert_eq!(cold["authored_project_json_utf8"],edited);
            assert!(call("connect",&edited,serde_json::json!({})).unwrap_err().contains("NEW_RUN_REQUIRED"));
            let fresh=call("new_run",&edited,serde_json::json!({"authoring_selection":saved["authoring_selection"],"seed":"0"})).unwrap();
            assert_eq!(fresh["first_frame_pending"],true);
            assert!(call("new_run",&edited,serde_json::json!({"authoring_selection":saved["authoring_selection"],"seed":"0"})).is_err());
            let next=call("run_to",&edited,serde_json::json!({"next_tick":"2"})).unwrap();
            assert_eq!(next["summary"]["output_display"]["texts"],serde_json::json!([amount.to_string(),amount.to_string()]));
            let committed=call("save_world",&edited,serde_json::json!({"next_tick":"2","state_hash":next["summary"]["state_hash"]})).unwrap();
            // A catalogue can still contain the pre-edit source after the
            // edited world has become current. Load only returns its source;
            // stale save/connect/run must keep rejecting that old source.
            let reopened=call("load",&original,serde_json::json!({})).unwrap();
            assert_eq!(reopened["kind"],"native_source_current");
            assert_eq!(reopened["authored_project_json_utf8"],edited);
            assert_eq!(reopened["requires_new_run"],false);
            assert!(reopened.get("snapshot_hex").is_none());
            for operation in ["save_source","connect","run"] {
                let fields=if operation=="run"{serde_json::json!({"ticks":1})}else{serde_json::json!({})};
                assert!(call(operation,&original,fields).unwrap_err().contains("SOURCE_CHANGED"));
            }
            selected=committed["authoring_selection"].clone();previous=edited;
        }
    }
    #[test]
    #[cfg(windows)]
    fn actual_controller_source_edit_and_connect_reuse_one_world(){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let source=SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { (매마디)마다 { 이름표 <- (이름=\"첫\")인 명찰. 이름표 보여주기. }. }.".into()}]};
        let original=ProjectArtifact::create("v1-core-v25".into(),source.clone(),String::new()).unwrap().save().unwrap();
        let controller=NativeSourceController::start();
        let call=|operation:&str,authored:&str|controller.call(serde_json::json!({"operation":operation,"authored_project_json_utf8":authored}).to_string());
        call("save_source",&original).unwrap();
        let mut changed=source;changed.files[1].text=changed.files[1].text.replace("첫","둘");
        let edited=crate::authored_project::operate("update",&serde_json::json!({"original_json":original,"source_snapshot":changed}).to_string()).unwrap();
        call("save_source",&edited).unwrap();
        assert!(call("connect",&original).unwrap_err().contains("SOURCE_CHANGED"));
        let first:serde_json::Value=serde_json::from_str(&call("connect",&edited).unwrap()).unwrap();
        assert_eq!(first["first_frame_pending"],true);
        crate::input_receipt::parse_receipt_snapshot(&first["first_input_receipt"]).unwrap();
        let second:serde_json::Value=serde_json::from_str(&call("connect",&edited).unwrap()).unwrap();
        assert_eq!(second["first_frame_pending"],false);
        assert_eq!(second["snapshot_hex"],first["snapshot_hex"]);
        assert_eq!(second["saved_artifact_json_utf8"],first["saved_artifact_json_utf8"]);
        let before=call("load",&edited).unwrap();
        let no_op:serde_json::Value=serde_json::from_str(&call("save_source",&edited).unwrap()).unwrap();
        assert_eq!(no_op["kind"],"native_source_saved");
        assert_eq!(no_op["authored_project_json_utf8"],edited);
        assert_eq!(call("load",&edited).unwrap(),before,"source no-op must not publish a new world or receipt");
        assert!(call("save_source",&original).unwrap_err().contains("SOURCE_CHANGED"));
        assert_eq!(call("load",&edited).unwrap(),before,"different source must preserve the committed world");
    }
}

#[cfg(windows)]
fn request_authored(request:&Request)->Option<&str>{
    match request {
        Request::GeneralRecordingContent{authored_project_json_utf8}
        |Request::LogicalNewRun{authored_project_json_utf8,..}|Request::LogicalSave{authored_project_json_utf8,..}
        |Request::LogicalLoad{authored_project_json_utf8,..}|Request::LogicalVerifyReceipt{authored_project_json_utf8,..}
        |Request::LogicalFinishBegin{authored_project_json_utf8,..}|Request::LogicalFinishRecover{authored_project_json_utf8,..}
        |Request::LogicalNewRecord{authored_project_json_utf8,..}|Request::Save{authored_project_json_utf8,..}
        |Request::Load{authored_project_json_utf8,..}|Request::NewRun{authored_project_json_utf8,..}
        |Request::Run{authored_project_json_utf8,..}|Request::Connect{authored_project_json_utf8,..}
        |Request::RunTo{authored_project_json_utf8,..}|Request::SaveWorld{authored_project_json_utf8,..}=>Some(authored_project_json_utf8),
        _=>None,
    }
}
#[cfg(windows)]
fn dispatch_logic_source(request:&Request,authored:&str,project:&crate::authored_project::ProjectArtifact)->Result<String,String>{
    use crate::runtime_surface as runtime;
    let is_save=match request {
        Request::Save{previous_authored_project_json_utf8,..}=>{
            if previous_authored_project_json_utf8.as_deref().is_some_and(|prior|prior!=authored){
                return Err("E_SOURCE_SAVE_PARENT: B1 Native는 최초 저장만 지원하며 원문 갱신은 지원하지 않습니다".into());
            }true
        },
        Request::Load{..}=>false,
        // Existing B1 approval is source-only. Refuse before reading/consuming
        // any first-use permit; this patch does not enable Native worlds.
        _=>return Err("E_SOURCE_FIRST_USE_UNSUPPORTED: B1 Native는 원문 저장·재열기만 지원하며 세계 실행은 지원하지 않습니다".into()),
    };
    let project_id=project.authored_identity.logical_dependency_id.as_str();
    let path=root(project_id)?;let locator_path=path.join("source.locator.json");
    if !locator_path.exists(){
        if !is_save{return Err("E_NATIVE_SOURCE_SAVE_REQUIRED: 먼저 현재 원문을 저장하세요".into());}
        let revision=revision()?;
        let raw=crate::authored_project::operate("source_prepare",&serde_json::json!({
            "artifact_json_utf8":authored,"expected_parent_hex":"","revision_hex":hex::encode(&revision),
            "previous_source_artifact_utf8":null}).to_string())?;
        let source=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::from_artifact_logic_private(&raw)?;
        if source.project()!=identity(project_id)? || source.owned().project().save()?!=authored{
            return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());
        }
        // Every full source/Bool/dependency check above precedes provisioning.
        fs::create_dir(&path).map_err(|error|format!("E_NATIVE_SOURCE_STORE_EXISTS_OR_UNAVAILABLE: {error}"))?;
        let actual=runtime::save_initial_prepared_logic_source_native(&source,&path)?;
        let locator=SourceLocator{project_hex:hex::encode(&actual.project_identity),source_revision_hex:hex::encode(&actual.revision_identity),
            source_integrity_hex:hex::encode(actual.revision_integrity),grant_locator:actual.grant_locator.ok_or("E_NATIVE_SOURCE_GRANT_MISSING")?,
            authored_project_json_utf8:authored.into()};write_once(&locator_path,&locator)?;
    }
    let locator:SourceLocator=read(&locator_path)?;
    if locator.project_hex!=project_id{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());}
    let (actual,recorded)=runtime::read_authored_source_native_recorded(&path,&identity(project_id)?,&locator.grant_locator)?;
    let source_artifact_utf8=recorded.ok_or("E_LOGIC_STORAGE_CONTEXT_REQUIRED")?;
    if hex::encode(&actual.revision_identity)!=locator.source_revision_hex || hex::encode(actual.revision_integrity)!=locator.source_integrity_hex{
        return Err("E_NATIVE_SOURCE_SOURCE_CURRENT".into());
    }
    let payload:serde_json::Value=serde_json::from_str(&actual.project_payload_utf8).map_err(|_|"E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
    let current=payload["authored_project_json_utf8"].as_str().ok_or("E_NATIVE_SOURCE_SOURCE_PAYLOAD")?;
    if is_save && current!=authored{return Err("E_SOURCE_SAVE_PARENT: B1 Native 원문 갱신은 지원하지 않습니다".into());}
    // Existing response only. Proposed source_artifact_utf8 is not adopted.
    Ok(serde_json::json!({"kind":if is_save{"native_source_saved"}else{"native_source_current"},
        "authored_project_json_utf8":current,"source_artifact_utf8":source_artifact_utf8}).to_string())
}

#[cfg(windows)]
const PUBLIC_CALLABLE_LOCATOR: &str = "public-callable.locator.json";

#[cfg(windows)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicCallableLocator {
    project_hex: String,
    revision_hex: String,
    integrity_hex: String,
}

#[cfg(windows)]
fn save_public_callable_source(raw: &str) -> Result<String, String> {
    use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision;

    let source = PreparedSourceOnlyRevision::from_artifact(raw)?;
    if source.owned().sources().public_callable_artifact().is_none() {
        return Err("E_DDL_V4_RECORDED_CONTEXT_REQUIRED".into());
    }
    if !source.parent().is_empty() {
        return Err("E_SOURCE_SAVE_PARENT: 첫 원문 저장에는 빈 부모만 허용합니다".into());
    }
    let project_hex = hex::encode(source.project());
    let path = root(&project_hex)?;
    let locator_path = path.join(PUBLIC_CALLABLE_LOCATOR);
    if locator_path.is_file() {
        let existing = read_public_callable_source(&path, &project_hex)?;
        if existing != raw {
            return Err("E_NATIVE_SOURCE_SOURCE_CHANGED".into());
        }
    } else {
        // An occupied root could be another source owner or an unconfirmed commit.
        fs::create_dir(&path).map_err(|e| format!("E_NATIVE_SOURCE_STORE_EXISTS_OR_UNAVAILABLE: {e}"))?;
        let request = serde_json::json!({"source_artifact_utf8": raw, "storage_root": path});
        crate::authored_project::operate("public_callable_source_save_native", &request.to_string())?;
        let locator = PublicCallableLocator {
            project_hex: project_hex.clone(),
            revision_hex: hex::encode(source.revision()),
            integrity_hex: hex::encode(source.integrity()),
        };
        write_once(&locator_path, &locator)?;
    }
    Ok(serde_json::json!({"kind":"public_callable_source_saved", "project_hex":project_hex}).to_string())
}

#[cfg(windows)]
fn read_public_callable_source(path: &Path, project_hex: &str) -> Result<String, String> {
    use crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;

    let locator: PublicCallableLocator = read(&path.join(PUBLIC_CALLABLE_LOCATOR))?;
    if locator.project_hex != project_hex {
        return Err("E_NATIVE_SOURCE_READBACK_PROJECT".into());
    }
    let integrity: [u8; 32] = identity(&locator.integrity_hex)?
        .try_into().map_err(|_| "E_NATIVE_SOURCE_LOCATOR_IDENTITY")?;
    let adapter = SqliteAdapterV1::open_source_revision(path)
        .map_err(|e| format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let stored = adapter.load_current_source_revision(
        &identity(project_hex)?, &identity(&locator.revision_hex)?, &integrity,
    ).map_err(|e| format!("E_SOURCE_REVISION_READBACK: {e:?}"))?;
    let source = stored.source();
    if source.owned().sources().public_callable_artifact().is_none() {
        return Err("E_DDL_V4_RECORDED_CONTEXT_REQUIRED".into());
    }
    source.artifact()
}

#[cfg(windows)]
fn load_public_callable_source(project_hex: &str) -> Result<String, String> {
    let path = root(project_hex)?;
    let raw = read_public_callable_source(&path, project_hex)?;
    Ok(serde_json::json!({
        "kind":"public_callable_source_current", "project_hex":project_hex, "source_artifact_utf8":raw
    }).to_string())
}

#[cfg(windows)]
fn prepare_workbench_consumer(original:&str,create:&str,path:&str)->Result<String,String>{
    if create.len()>1_048_576{return Err("E_SOURCE_SAVE_REQUEST_LIMIT".into());}
    let rejected=|(message,diagnostic):(String,Option<crate::preprocess::ExecutionDiagnostic>)|
        serde_json::json!({"kind":"public_callable_consumer_rejected","message":message,
            "diagnostic":diagnostic.map(|row|row.to_json_value())}).to_string();
    let project=match crate::authored_project::operate_for_cli("create",create){
        Ok(project)=>project,
        Err(error)=>return Ok(rejected(error)),
    };
    let input=serde_json::json!({"original_record_json_utf8":original,
        "consumer_project_json_utf8":project,"consumer_source_path":path,"revision_hex":hex::encode(revision()?)});
    match crate::authored_project::operate_for_cli("public_callable_source_prepare",&input.to_string()){
        Ok(artifact)=>Ok(serde_json::json!({"kind":"public_callable_consumer_prepared",
            "source_artifact_utf8":artifact}).to_string()),
        Err(error)=>Ok(rejected(error)),
    }
}

#[cfg(windows)]
type EditionSource=crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision;
#[cfg(windows)]
type EditionAdapter=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1;

#[cfg(windows)]
fn selected_public_edition(project_hex:&str,selection:EditionSelection)
    ->Result<(EditionAdapter,EditionSource,bool),String>{
    let path=root(project_hex)?;
    let anchor:PublicCallableLocator=read(&path.join(PUBLIC_CALLABLE_LOCATOR))?;
    if anchor.project_hex!=project_hex{return Err("E_NATIVE_SOURCE_READBACK_PROJECT".into());}
    let integrity=identity(&anchor.integrity_hex)?.try_into().map_err(|_|"E_NATIVE_SOURCE_LOCATOR_IDENTITY")?;
    let adapter=EditionAdapter::open_source_revision(&path).map_err(|e|format!("E_SOURCE_REVISION_OPEN: {e:?}"))?;
    let (initial,head)=adapter.read_public_editions_at_anchor(
        &identity(project_hex)?,&identity(&anchor.revision_hex)?,&integrity,
    ).map_err(|e|format!("E_DDL_EDITION_READBACK: {e:?}"))?;
    let updated=head.is_some();
    let source=match selection{
        EditionSelection::Current=>head.unwrap_or(initial),
        EditionSelection::Previous if updated=>initial,
        EditionSelection::Previous=>return Err("E_DDL_EDITION_REVISION_SELECTION".into()),
    };
    if source.owned().sources().public_callable_artifact().is_none(){return Err("E_DDL_V4_RECORDED_CONTEXT_REQUIRED".into());}
    Ok((adapter,source,updated))
}

#[cfg(windows)]
fn public_edition_view(source:&EditionSource,has_previous:bool)->Result<String,String>{
    let project=source.owned().sources().public_callable_artifact().ok_or("E_DDL_V4_RECORDED_CONTEXT_REQUIRED")?;
    Ok(serde_json::json!({
        "kind":"public_callable_edition_view","revision_hex":hex::encode(source.revision()),
        "project_hex":hex::encode(source.project()),"has_previous":has_previous,
        "authoring":project.edition_authoring_view()?,
    }).to_string())
}

#[cfg(windows)]
fn list_public_callable_sources() -> Result<String, String> {
    let mut projects = Vec::new();
    match fs::read_dir(local_source_base()?) {
        Ok(entries) => for entry in entries {
            let entry = entry.map_err(|_| "E_NATIVE_SOURCE_LOCATOR_READ")?;
            let id = entry.file_name().to_string_lossy().into_owned();
            if id.len() == 64 && identity(&id).is_ok()
                && entry.file_type().map_err(|_| "E_NATIVE_SOURCE_LOCATOR_READ")?.is_dir()
                && entry.path().join(PUBLIC_CALLABLE_LOCATOR).is_file()
            {
                if projects.len() == 256 {
                    return Err("E_NATIVE_SOURCE_LOCATOR_LIMIT".into());
                }
                projects.push(id);
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(_) => return Err("E_NATIVE_SOURCE_LOCATOR_READ".into()),
    }
    projects.sort_unstable();
    Ok(serde_json::json!({"kind":"public_callable_source_list","projects":projects}).to_string())
}
