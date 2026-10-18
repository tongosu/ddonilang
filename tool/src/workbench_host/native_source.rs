//! Physical same-origin Native host transport. JSON is only a locator/request;
//! all execution authority stays in the protected Native source owner.
use std::{collections::BTreeMap,fs,io::Write,path::{Path,PathBuf},sync::mpsc};
use serde::{Deserialize,Serialize};

#[derive(Debug,Clone)]
pub(super) struct NativeSourceController {sender:mpsc::SyncSender<(String,mpsc::Sender<Result<String,String>>)>}
impl NativeSourceController {
    pub(super) fn start()->Self{
        let (sender,receiver)=mpsc::sync_channel::<(String,mpsc::Sender<Result<String,String>>) >(2);
        // Runtime owners may contain Rc. Construct and use them only here,
        // never move a VM/permit through the HTTP worker threads.
        std::thread::spawn(move||{
            #[cfg(windows)] let mut retained=BTreeMap::new();
            for (request,reply) in receiver {
                #[cfg(windows)] let result=dispatch(&request,&mut retained);
                #[cfg(not(windows))] let result={let _=request;Err("E_SOURCE_FIRST_USE_UNSUPPORTED: Native 보호 실행을 지원하지 않습니다".into())};
                let _=reply.send(result);
            }
        });Self{sender}
    }
    pub(super) fn call(&self,request:String)->Result<String,String>{
        let (sender,receiver)=mpsc::channel();
        self.sender.try_send((request,sender)).map_err(|_|"E_NATIVE_SOURCE_BUSY_OR_CLOSED")?;
        receiver.recv().map_err(|_|"E_NATIVE_SOURCE_SESSION_CLOSED")?
    }
}
#[derive(Deserialize)]
#[serde(tag="operation",deny_unknown_fields)]
enum Request {
    #[serde(rename="save_source")] Save {authored_project_json_utf8:String},
    #[serde(rename="run")] Run {authored_project_json_utf8:String,ticks:u32},
    #[serde(rename="load")] Load {authored_project_json_utf8:String},
    #[serde(rename="connect")] Connect {authored_project_json_utf8:String},
    #[serde(rename="run_to")] RunTo {authored_project_json_utf8:String,next_tick:String},
    #[serde(rename="save_world")] SaveWorld {authored_project_json_utf8:String,next_tick:String,state_hash:String},
}
#[derive(Serialize,Deserialize)]
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
fn identity(text:&str)->Result<Vec<u8>,String>{
    if text.is_empty()||text.len()>8192||text.len()%2!=0||!text.bytes().all(|v|v.is_ascii_digit()||(b'a'..=b'f').contains(&v)) {
        return Err("E_NATIVE_SOURCE_LOCATOR_IDENTITY".into());
    }hex::decode(text).map_err(|_|"E_NATIVE_SOURCE_LOCATOR_IDENTITY".into())
}
fn revision()->Result<Vec<u8>,String>{let mut bytes=[0;16];getrandom::fill(&mut bytes).map_err(|_|"E_NATIVE_SOURCE_RANDOM")?;Ok(bytes.to_vec())}
fn root(project:&str)->Result<PathBuf,String>{
    if project.len()!=64{return Err("E_NATIVE_SOURCE_PROJECT_IDENTITY".into());}identity(project)?;
    let base=std::env::var_os("LOCALAPPDATA").ok_or("E_NATIVE_SOURCE_LOCAL_ROOT")?;
    let base=PathBuf::from(base).join("Ddonilang").join("WorkbenchSource");
    fs::create_dir_all(&base).map_err(|e|format!("E_NATIVE_SOURCE_LOCAL_ROOT: {e}"))?;
    Ok(base.join(project))
}

#[cfg(windows)]
fn dispatch(text:&str,retained:&mut BTreeMap<String,crate::runtime_surface::NativeFirstSourceWorldRetained>)->Result<String,String>{
    use crate::runtime_surface as runtime;
    let request:Request=serde_json::from_str(text).map_err(|_|"E_NATIVE_SOURCE_REQUEST_FIELDS")?;
    let (authored,ticks,is_save)=match &request {
        Request::Save{authored_project_json_utf8}=>(authored_project_json_utf8,0,true),
        Request::Run{authored_project_json_utf8,ticks}=>(authored_project_json_utf8,*ticks,false),
        Request::Load{authored_project_json_utf8}=>(authored_project_json_utf8,0,false),
        Request::Connect{authored_project_json_utf8}=>(authored_project_json_utf8,0,false),
        Request::RunTo{authored_project_json_utf8,..}|Request::SaveWorld{authored_project_json_utf8,..}=>(authored_project_json_utf8,0,false),
    };
    if matches!(request,Request::Run{..})&&(ticks==0||ticks>2000){return Err("E_NATIVE_SOURCE_MADI_LIMIT: 한 번에 1~2000마디를 지정하세요".into());}
    let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())?;
    let project=owned.project().authored_identity.logical_dependency_id.as_str().to_owned();
    let path=root(&project)?;let locator_path=path.join("source.locator.json");
    if !locator_path.exists(){
        if !is_save{return Err("E_NATIVE_SOURCE_SAVE_REQUIRED: 먼저 현재 원문을 저장하세요".into());}
        // An existing store without a locator is uncertain, never reprovision.
        fs::create_dir(&path).map_err(|e|format!("E_NATIVE_SOURCE_STORE_EXISTS_OR_UNAVAILABLE: {e}"))?;
        let source=runtime::save_initial_authored_source_native(authored,&revision()?,&path)?;
        let locator=SourceLocator{project_hex:hex::encode(source.project_identity),source_revision_hex:hex::encode(source.revision_identity),
            source_integrity_hex:hex::encode(source.revision_integrity),grant_locator:source.grant_locator.ok_or("E_NATIVE_SOURCE_GRANT_MISSING")?,
            authored_project_json_utf8:authored.clone()};
        write_once(&locator_path,&locator)?;
    }
    let mut source:SourceLocator=read(&locator_path)?;
    if source.project_hex!=project {
        return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 저장된 원문과 다릅니다. 기존 저장본은 보존했습니다".into());
    }
    let world_path=path.join("world.locator.json");
    if world_path.exists(){
        if is_save{return Err("E_NATIVE_SOURCE_FOLLOWUP_SAVE_UNSUPPORTED: 기존 세계 저장본은 보존했습니다. 후속 세계 저장 연결은 아직 지원하지 않습니다".into());}
        let digest=if let Some(owner)=retained.get_mut(&project){owner.current_digest(&identity(&project)?,&source.grant_locator)?}
            else{runtime::native_source_current_digest(&path,&identity(&project)?,&source.grant_locator)?};
        let world:WorldLocator=read(&path.join(format!("world-{}.json",hex::encode(digest))))?;
        source.source_revision_hex=world.parent_hex.clone();
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(world.saved_artifact_json_utf8.as_bytes())?;
        let saved_owner=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
        if saved_owner.project().save()?!=*authored{return Err("E_NATIVE_SOURCE_SOURCE_CHANGED: 저장된 세계의 원문과 다릅니다".into());}
        if matches!(request,Request::Connect{..}) {
            let parent=identity(&source.source_revision_hex)?;let revision=identity(&world.revision_hex)?;
            let snapshot=if let Some(owner)=retained.get_mut(&project){owner.browser_snapshot(&parent,&revision,&source.grant_locator)?}
                else{runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,&parent,&revision,&path,&source.grant_locator)?};
            return Ok(serde_json::json!({"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
                "snapshot_hex":hex::encode(snapshot),"parent_hex":source.source_revision_hex,"revision_hex":world.revision_hex,"first_frame_pending":false}).to_string());
        }
        let ticks=match &request{
            Request::RunTo{next_tick,..}|Request::SaveWorld{next_tick,..}=>{
                let next:u64=next_tick.parse().map_err(|_|"E_NATIVE_SOURCE_MADI_IDENTITY")?;
                if next.to_string()!=*next_tick{return Err("E_NATIVE_SOURCE_MADI_IDENTITY".into());}
                let snapshot=if let Some(owner)=retained.get_mut(&project){owner.browser_snapshot(&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&source.grant_locator)?}
                    else{runtime::native_source_browser_snapshot(&world.saved_artifact_json_utf8,&identity(&world.parent_hex)?,&identity(&world.revision_hex)?,&path,&source.grant_locator)?};
                let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(world.saved_artifact_json_utf8.as_bytes())?;
                let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes)?;
                let pesi=crate::project_snapshot_identity::current::issue_saved_local_project(&owned)?;
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
                return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_world_saved","summary":saved})).compact_text());
            }
            return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_world_saved","summary":result})).compact_text());
        }
        // Cold result is not a new durable checkpoint. Keep its exact scope.
        return Ok(crate::unit_wire::SnapshotJsonOwner::from_value(serde_json::json!({"kind":"native_cold_result",
            "authored_project_json_utf8":authored,"summary":result,"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
            "continued_state_saved":false})).compact_text());
    }
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
    let next=revision()?;
    let run=session.execute(if matches!(request,Request::Connect{..}){1}else{ticks},0,None,None)?.prepare(&next)?.write_prepared()?.commit()?;
    let world=WorldLocator{parent_hex:source.source_revision_hex.clone(),revision_hex:hex::encode(next),saved_artifact_json_utf8:run.saved_runtime_artifact()?.into()};
    let report=run.report_json()?;
    // Keep the active pinned owner: a second adapter cannot acquire its lease.
    let mut run=run;
    let digest=run.current_digest(&identity(&project)?,&source.grant_locator)?;
    write_once(&path.join(format!("world-{}.json",hex::encode(digest))),&world)?;
    retained.insert(project,run); // Same protected owner survives the response.
    write_once(&world_path,&world)?;
    if matches!(request,Request::Connect{..}) {
        let owner=retained.get_mut(&source.project_hex).ok_or("E_NATIVE_SOURCE_SESSION_CLOSED")?;
        let snapshot=owner.browser_snapshot(&identity(&source.source_revision_hex)?,&identity(&world.revision_hex)?,&source.grant_locator)?;
        let receipt=owner.summary()?["input_receipts"].as_array().and_then(|rows|rows.last()).ok_or("E_NATIVE_SOURCE_INPUT_RECEIPT")?;
        return Ok(serde_json::json!({"saved_artifact_json_utf8":world.saved_artifact_json_utf8,
            "snapshot_hex":hex::encode(snapshot),"parent_hex":source.source_revision_hex,"revision_hex":world.revision_hex,"first_frame_pending":true,
            "first_input_receipt":receipt}).to_string());
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
        assert!(call("save_source",&edited).unwrap_err().contains("FOLLOWUP_SAVE_UNSUPPORTED"));
    }
}
