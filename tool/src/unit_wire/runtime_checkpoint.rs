//! Existing runtime checkpoint admission shared by Native and WASM.
//! This stages world/input/observation/view values, not a durable transaction
//! or an input recording. Source/revision custody is supplied by its owner.
use ddonirang_core::{Fixed64,Nuri,platform::DetNuri};
use ddonirang_lang::runtime::Value;
use serde_json::{json,Value as JsonValue};
use super::SnapshotJsonOwner as OwnedJson;
use std::collections::BTreeMap;

pub(crate) struct PublicObservationSurface {
    pub(crate) trace:OwnedJson,
    pub(crate) trace_hash:String,
    pub(crate) table_projection:OwnedJson,
}
pub(crate) struct RestoredInputState {
    pub(crate) keys_pressed:u64,
    pub(crate) last_key_name:String,
    pub(crate) pointer_x_i32:i32,
    pub(crate) pointer_y_i32:i32,
    pub(crate) rng_seed:u64,
    pub(crate) dt:Fixed64,
}
pub(crate) struct StagedRuntimeState {
    pub(crate) world:DetNuri,
    pub(crate) param_overrides:BTreeMap<String,Value>,
    pub(crate) tick_id:u64,
    pub(crate) input:RestoredInputState,
    pub(crate) public_observation_frames:OwnedJson,
    pub(crate) last_public_observations:OwnedJson,
    pub(crate) observation_surface:PublicObservationSurface,
    pub(crate) scene_history:crate::bogae_time::SceneHistory,
    pub(crate) bogae_projection:Option<OwnedJson>,
}
pub(crate) fn build_public_observation_surface(
    source_path:&str,source_bytes_sha256:&str,canonical_ddn_sha256:&str,frames:&[JsonValue],
)->Result<PublicObservationSurface,String>{
    let source_identity=format!("sha256:{source_bytes_sha256}");
    let revision_identity=format!("sha256:{canonical_ddn_sha256}");
    let run_id=format!("run:{}",crate::runtime_surface::sha256_hex(
        format!("{source_path}\0{source_identity}\0{revision_identity}\0{}",frames.len()).as_bytes()));
    let mut request=OwnedJson::from_value(json!({
        "project_id":"","run_id":run_id,"source_identity":source_identity,
        "revision_identity":revision_identity,"run_status":"completed","frames":null,
    }));
    request.value_mut()["frames"] = JsonValue::Array(frames.iter()
        .map(OwnedJson::clone_from_value).map(OwnedJson::into_parent_value).collect());
    let bundle=OwnedJson::from_value(crate::deterministic_debug::build_public_observation_trace(&request)?);
    let trace=OwnedJson::clone_from_value(bundle.get("trace")
        .ok_or("E_WASM_OBSERVATION_TRACE_MISSING: trace가 없습니다")?);
    let trace_hash=bundle.get("trace_hash").and_then(JsonValue::as_str)
        .ok_or("E_WASM_OBSERVATION_TRACE_HASH_MISSING: trace hash가 없습니다")?.to_string();
    let table_projection=OwnedJson::from_value(crate::deterministic_debug::build_public_observation_table_projection(&trace,&run_id)?);
    Ok(PublicObservationSurface{trace,trace_hash,table_projection})
}
fn uint(value:&JsonValue)->Option<u64>{match value{
    JsonValue::Number(n)=>n.as_u64(),JsonValue::String(s)=>s.parse().ok(),_=>None,
}}
fn int(value:&JsonValue)->Option<i32>{match value{
    JsonValue::Number(n)=>n.as_i64().and_then(|n|i32::try_from(n).ok()),
    JsonValue::String(s)=>s.parse().ok(),_=>None,
}}
fn validate_committed_madi_cursor(frames:&[JsonValue],next_tick:u64)->Result<(),String>{
    let invalid=||"E_WASM_STATE_TICK_INVALID: next_tick_id와 실제 처리한 관찰 마디가 일치하지 않습니다".to_string();
    let mut previous=None;
    for frame in frames {
        let madi=frame.get("committed_madi").and_then(uint).ok_or_else(invalid)?;
        if previous.is_some_and(|prior|madi<=prior){return Err(invalid());}
        previous=Some(madi);
    }
    let expected=match previous {Some(last)=>last.checked_add(1).ok_or_else(invalid)?,None=>0};
    if next_tick!=expected{return Err(invalid());}
    Ok(())
}
fn parse_input(payload:Option<&JsonValue>)->Result<RestoredInputState,String>{
    let payload=payload.ok_or("E_INPUT_RECORD_LEGACY_UNSUPPORTED: 입력 기록이 없어 정확한 dt를 확인할 수 없습니다")?;
    let object=payload.as_object().ok_or("E_WASM_STATE_INPUT_INVALID: state.input은 object여야 합니다")?;
    let u=|key:&str|object.get(key).map(|v|uint(v).ok_or_else(||format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}"))).transpose();
    let missing=|key:&str|format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}가 없어 처리한 입력을 복원할 수 없습니다");
    let i=|key:&str|object.get(key).ok_or_else(||missing(key))
        .and_then(|v|int(v).ok_or_else(||format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}")));
    let last_key_name=match object.get("last_key_name"){
        Some(JsonValue::String(v))=>v.clone(),None=>return Err(missing("last_key_name")),
        _=>return Err("E_WASM_STATE_INPUT_INVALID: state.input.last_key_name은 string이어야 합니다".into()),
    };
    Ok(RestoredInputState{keys_pressed:u("keys_pressed")?.ok_or_else(||missing("keys_pressed"))?,last_key_name,
        pointer_x_i32:i("pointer_x_i32")?,pointer_y_i32:i("pointer_y_i32")?,
        // A processed frame's rng_seed may already be tick-derived. It cannot
        // replace the recorded base seed used to derive subsequent frames.
        rng_seed:u("rng_base_seed")?.ok_or_else(||missing("rng_base_seed"))?,dt:crate::input_receipt::parse_recorded_dt(payload)?})
}
pub(crate) fn stage_runtime_state(
    payload:&JsonValue,source_path:&str,source_bytes_sha256:&str,canonical_ddn_sha256:&str,
)->Result<StagedRuntimeState,String>{
    stage_runtime_state_with_tuple_plan(payload,source_path,source_bytes_sha256,canonical_ddn_sha256,None)
}

pub(crate) fn stage_runtime_state_with_tuple_plan(
    payload:&JsonValue,source_path:&str,source_bytes_sha256:&str,canonical_ddn_sha256:&str,
    plan:Option<&crate::ddn_runtime::SavedTupleRestorePlan>,
)->Result<StagedRuntimeState,String>{
    stage_runtime_state_with_catalog(payload,source_path,source_bytes_sha256,canonical_ddn_sha256,plan,None)
}
pub(crate) fn stage_runtime_state_with_catalog(
    payload:&JsonValue,source_path:&str,source_bytes_sha256:&str,canonical_ddn_sha256:&str,
    plan:Option<&crate::ddn_runtime::SavedTupleRestorePlan>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
)->Result<StagedRuntimeState,String>{
    stage_runtime_state_with_complete_frame(payload,source_path,source_bytes_sha256,canonical_ddn_sha256,plan,catalog,None)
}
pub(crate) fn stage_runtime_state_with_complete_frame(
    payload:&JsonValue,source_path:&str,source_bytes_sha256:&str,canonical_ddn_sha256:&str,
    plan:Option<&crate::ddn_runtime::SavedTupleRestorePlan>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
    frame:Option<super::CompleteStateFrameV1<'_>>,
)->Result<StagedRuntimeState,String>{
    let state=payload.get("state").and_then(JsonValue::as_object)
        .ok_or("E_WASM_STATE_CHANNEL_MISSING: state object가 필요합니다")?;
    if state.contains_key("snapshot_v1"){return Err("E_WASM_STATE_SNAPSHOT_MIXED_VERSION: snapshot_v1은 snapshot_v2와 함께 읽지 않습니다".into());}
    let snapshot=state.get("snapshot_v2").and_then(JsonValue::as_object)
        .ok_or("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH: snapshot_v2가 필요합니다")?;
    for (key,expected,code) in [
        ("schema","ddn.wasm_state_snapshot.v2","E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH"),
        ("resource_snapshot_schema",super::RESOURCE_SNAPSHOT_SCHEMA_V2,"E_RESOURCE_SNAPSHOT_VERSION_MISMATCH"),
        ("unit_wire_schema",super::UNIT_VALUE_WIRE_SCHEMA_V2,"E_UNIT_WIRE_CHECKPOINT_MISMATCH"),
    ]{
        let actual=snapshot.get(key).and_then(JsonValue::as_str);
        if actual!=Some(expected){return Err(format!("{code}: expected {expected}, got {}",actual.unwrap_or("<missing>")));}
    }
    let resources=snapshot.get("resources").ok_or("E_RESOURCE_SNAPSHOT_MISSING: snapshot_v2.resources")?;
    let mut world=DetNuri::new();
    if let Some(frame)=frame{*world.world_mut()=super::resource_snapshot::stage_resource_snapshot_with_complete_frame(resources,&frame,plan,catalog)?;}
    else{super::resource_snapshot::restore_world_resources_with_catalog(world.world_mut(),resources,plan,catalog)?;}
    let param_overrides=super::parse_param_overrides(snapshot.get("param_overrides")
        .ok_or("E_WASM_STATE_PARAMS_MISSING: snapshot_v2.param_overrides")?)?;
    let frames=snapshot.get("public_observation_frames").filter(|value|value.is_array())
        .ok_or("E_WASM_STATE_OBSERVATION_FRAMES_INVALID: public_observation_frames array가 필요합니다")?;
    // Keep deep saved observations under the shared heap owner throughout all
    // later validation. A late input/scene/display rejection must not recursively
    // clone or destroy previously staged rows on the Native/WASM call stack.
    let public_observation_frames=OwnedJson::clone_from_value(frames);
    let tick_id=snapshot.get("next_tick_id").and_then(uint).ok_or("E_WASM_STATE_TICK_INVALID: next_tick_id가 필요합니다")?;
    if let Some(frame)=frame{
        let parameters=param_overrides.iter().map(|(name,value)|match value{
            Value::Fixed64(value)=>Ok((name.clone(),*value)),
            _=>Err("E_VC04_COMPLETE_RESOURCE_BINDING: binary 공급이 없는 매개변수 씨입니다".to_owned()),
        }).collect::<Result<BTreeMap<_,_>,_>>()?;
        let encoded=super::complete_state_tables::encode_parameter_table(&parameters)
            .map_err(|cause|format!("E_VC04_COMPLETE_RESOURCE_BINDING: {cause:?}"))?;
        if frame.next_tick!=tick_id||encoded!=frame.parameters{
            return Err("E_VC04_COMPLETE_RESOURCE_BINDING: binary·runtime 마디/매개변수가 다릅니다".into());
        }
    }
    validate_committed_madi_cursor(public_observation_frames.as_array().unwrap(),tick_id)?;
    let input=parse_input(state.get("input"))?;
    let scene_history=crate::bogae_time::SceneHistory::from_snapshot(snapshot.get("scene_history"),tick_id)?;
    let mut bogae_projection=scene_history.restored_projection(tick_id.checked_sub(1))?.map(OwnedJson::from_value);
    if let Some(projection)=bogae_projection.as_mut(){
        crate::bogae_time::resolve_result_tables(projection.value_mut(),public_observation_frames.as_array().unwrap(),
            &format!("sha256:{source_bytes_sha256}"),&format!("sha256:{canonical_ddn_sha256}"))?;
    }
    let observation_surface=build_public_observation_surface(source_path,source_bytes_sha256,canonical_ddn_sha256,public_observation_frames.as_array().unwrap())?;
    let last_public_observations=public_observation_frames.as_array().unwrap().last()
        .and_then(|frame|frame.get("public_observations")).filter(|value|value.is_array())
        .map(OwnedJson::clone_from_value).unwrap_or_else(||OwnedJson::from_value(json!([])));
    Ok(StagedRuntimeState{world,param_overrides,tick_id,input,public_observation_frames,last_public_observations,
        observation_surface,scene_history,bogae_projection})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beta1_staged_deep_observations_survive_success_and_late_rejection() {
        std::thread::Builder::new().stack_size(128*1024).spawn(|| {
            let mut deep=JsonValue::Bool(true);
            for _ in 0..20_000 { deep=JsonValue::Array(vec![deep]); }
            let deep=OwnedJson::from_value(deep);
            let mut payload=OwnedJson::from_value(json!({"state":{
                "input":input(),"snapshot_v2":{
                    "schema":"ddn.wasm_state_snapshot.v2",
                    "resource_snapshot_schema":super::super::RESOURCE_SNAPSHOT_SCHEMA_V2,
                    "unit_wire_schema":super::super::UNIT_VALUE_WIRE_SCHEMA_V2,
                    "resources":{"schema":super::super::RESOURCE_SNAPSHOT_SCHEMA_V2,
                        "unit_wire_schema":super::super::UNIT_VALUE_WIRE_SCHEMA_V2,
                        "components":[],"next_entity":0,"json":{},"fixed64":{},"handle":{},"value_det":{}},
                    "param_overrides":{},"next_tick_id":1,
                    "public_observation_frames":[{"committed_madi":0,"state_hash":"state-0",
                        "input_snapshot":null,"public_observations":[{"name":"값","dtype":"fixed64","unit":"",
                            "format":"ddn-canonical","source_ref":"main.ddn#bytes=0:3","value_canon":"1",
                            "value_text":null,"starts_observation_event":true}]}]
            }}}));
            payload.value_mut()["state"]["snapshot_v2"]["public_observation_frames"][0]["input_snapshot"]=
                deep.clone().into_parent_value();
            payload.value_mut()["state"]["snapshot_v2"]["public_observation_frames"][0]["public_observations"][0]["value_text"]=
                deep.clone().into_parent_value();
            let before=payload.compact_text();
            let staged=stage_runtime_state(&payload,"main.ddn","source","canon").unwrap();
            assert!(deep.equals_json(&staged.public_observation_frames[0]["input_snapshot"]));
            assert!(deep.equals_json(&staged.last_public_observations[0]["value_text"]));
            assert!(deep.equals_json(&staged.observation_surface.trace["samples"][0]["cells"][0]["value_text"]));
            assert_eq!(payload.compact_text(),before);
            drop(staged); // Normal destruction must be heap-owned too.
            payload.value_mut()["state"]["input"].as_object_mut().unwrap().remove("rng_base_seed");
            let rejected_before=payload.compact_text();
            assert!(stage_runtime_state(&payload,"main.ddn","source","canon").err().unwrap()
                .starts_with("E_WASM_STATE_INPUT_INVALID"));
            assert_eq!(payload.compact_text(),rejected_before);
            payload.value_mut()["state"]["input"]["rng_base_seed"]=json!(7);
            payload.value_mut()["state"]["snapshot_v2"]["public_observation_frames"][0]["public_observations"][0]
                .as_object_mut().unwrap().remove("value_canon");
            assert!(stage_runtime_state(&payload,"main.ddn","source","canon").is_err());
            drop(payload);drop(deep);
        }).unwrap().join().unwrap();
    }
    #[test]
    fn restored_observation_surface_preserves_deep_values_and_late_errors() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut deep=JsonValue::Bool(true);
            for _ in 0..20_000 { deep=JsonValue::Array(vec![deep]); }
            let deep=OwnedJson::from_value(deep);
            let mut frame=OwnedJson::from_value(json!({"committed_madi":0,"state_hash":"state-0",
                "input_snapshot":null,"public_observations":[{"name":"값","dtype":"fixed64","unit":"",
                    "format":"ddn-canonical","source_ref":"main.ddn#bytes=0:3","value_canon":"1",
                    "value_text":null,"starts_observation_event":true}]}));
            frame.value_mut()["input_snapshot"]=deep.clone().into_parent_value();
            frame.value_mut()["public_observations"][0]["value_text"]=deep.clone().into_parent_value();
            let frames=OwnedJson::from_value(JsonValue::Array(vec![frame.into_parent_value()]));
            let surface=build_public_observation_surface("main.ddn","source","canonical",frames.as_array().unwrap()).unwrap();
            let trace=surface.trace;
            let table=surface.table_projection;
            assert!(deep.equals_json(&trace["samples"][0]["cells"][0]["value_text"]));
            assert!(deep.equals_json(&table["tables"][0]["rows"][0]["cells"][0]["value_text"]));
            assert_eq!(surface.trace_hash,crate::deterministic_debug::public_observation_trace_hash(&trace));
            let mut invalid=frames.clone();
            invalid.value_mut().as_array_mut().unwrap().push(json!({"committed_madi":1}));
            assert!(build_public_observation_surface("main.ddn","source","canonical",invalid.as_array().unwrap())
                .err().unwrap().contains("state_hash"));
            drop(invalid);drop(table);drop(trace);drop(frames);drop(deep);
        }).unwrap().join().unwrap();
    }
    #[test]
    fn beta1_restore_madi_cursor_is_bound_to_committed_observations(){
        assert!(validate_committed_madi_cursor(&[],0).is_ok());
        assert!(validate_committed_madi_cursor(&[],1).is_err());
        let frames=vec![json!({"committed_madi":0}),json!({"committed_madi":1})];
        assert!(validate_committed_madi_cursor(&frames,2).is_ok());
        for next in [0,1,3,u64::MAX]{assert!(validate_committed_madi_cursor(&frames,next).is_err());}
        assert!(validate_committed_madi_cursor(&[frames[1].clone(),frames[0].clone()],2).is_err());
        assert!(validate_committed_madi_cursor(&[frames[0].clone(),frames[0].clone()],1).is_err());
        assert!(validate_committed_madi_cursor(&[json!({"committed_madi":u64::MAX})],0).is_err());
    }
    fn input()->JsonValue { json!({
        "keys_pressed":u64::MAX,"last_key_name":"왼쪽",
        "pointer_x_i32":i32::MIN,"pointer_y_i32":i32::MAX,
        "rng_base_seed":u64::MAX,"input_encoding":crate::input_receipt::SNAPSHOT_ENCODING,"dt":"1",
    }) }
    #[test]
    fn beta1_saved_input_never_defaults_missing_processed_fields(){
        let original=input();
        for field in ["keys_pressed","last_key_name","pointer_x_i32","pointer_y_i32","rng_base_seed"] {
            let mut damaged=original.clone();damaged.as_object_mut().unwrap().remove(field);
            assert!(parse_input(Some(&damaged)).is_err(),"missing {field} must not become zero/empty");
        }
    }
    #[test]
    fn beta1_saved_input_preserves_explicit_zero_extremes_and_seed_roles(){
        let original=input();let restored=parse_input(Some(&original)).unwrap();
        assert_eq!(restored.keys_pressed,u64::MAX);assert_eq!(restored.last_key_name,"왼쪽");
        assert_eq!(restored.pointer_x_i32,i32::MIN);assert_eq!(restored.pointer_y_i32,i32::MAX);
        assert_eq!(restored.rng_seed,u64::MAX);
        let mut zero=original.clone();
        for key in ["keys_pressed","pointer_x_i32","pointer_y_i32","rng_base_seed"]{zero[key]=json!(0);}
        zero["last_key_name"]=json!("");
        let restored=parse_input(Some(&zero)).unwrap();assert_eq!(restored.rng_seed,0);assert_eq!(restored.keys_pressed,0);
        // Per-frame seeds do not reconstruct a missing base seed.
        zero.as_object_mut().unwrap().remove("rng_base_seed");zero["rng_seed"]=json!(19);
        assert!(parse_input(Some(&zero)).is_err());
        zero["rng_base_seed"]=json!(23);assert_eq!(parse_input(Some(&zero)).unwrap().rng_seed,23);
        zero["rng_base_seed"]=JsonValue::Null;assert!(parse_input(Some(&zero)).is_err());
    }
}
