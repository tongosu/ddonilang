//! Model custody has no source-first-world permission or world locator.
use super::*;
use crate::unit_wire::project_runtime_attachment::model_source::{checked_project,ModelSource};
pub(super) fn dispatch(request:&Request,authored:&str)->Result<String,String>{
    let project=checked_project(authored)?.ok_or("E_MODEL_SOURCE_REQUIRED")?;
    let id=project.authored_identity.logical_dependency_id.as_str();let root=root(id)?;
    let (save,previous,selection)=match request{
        Request::Save{previous_authored_project_json_utf8,model_source_selection,..}=>(true,previous_authored_project_json_utf8.as_deref(),model_source_selection.as_ref()),
        Request::Load{..}=>(false,None,None),
        _=>return Err("E_MODEL_SOURCE_NO_WORLD: 보관한 모형 원문은 명시적으로 새 분석·실행하세요. 세계 이어하기는 지원하지 않습니다".into()),
    };
    let fresh=!root.exists();
    if fresh&&!save{return Err("E_NATIVE_SOURCE_SAVE_REQUIRED: 먼저 모형 원문을 명시 저장하세요".into());}
    let mut adapter=if fresh{
        fs::create_dir(&root).map_err(|_|"E_MODEL_SOURCE_STORE_UNAVAILABLE")?;
        crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::provision_model_source(&root)
    }else{crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::open_model_source(&root)}
        .map_err(|e|format!("E_MODEL_SOURCE_STORAGE_VERSION_OR_ACCESS: {e:?}"))?;
    let stored:ModelSource=if save{
        let next:[u8;16]=revision()?.try_into().map_err(|_|"E_MODEL_SOURCE_REVISION")?;
        adapter.save_model_source(authored,previous,selection,next,fresh)?
    }else{adapter.read_model_source(&hex::decode(id).map_err(|_|"E_MODEL_SOURCE_PROJECT")?.try_into().map_err(|_|"E_MODEL_SOURCE_PROJECT")?)?};
    if stored.profile!=project.source_profile{return Err("E_MODEL_SOURCE_PROFILE".into());}
    Ok(serde_json::json!({"kind":if save{"native_source_saved"}else{"native_source_current"},
        "source_kind":"MODEL_SOURCE_ONLY","model_source_selection":stored.anchor()?,"authored_project_json_utf8":stored.authored_project_json_utf8,
        "source_artifact_utf8":stored.encode()?,"requires_new_run":true}).to_string())
}
