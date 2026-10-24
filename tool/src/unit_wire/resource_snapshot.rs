//! One Native/WASM resource restore gate. Never skip a malformed channel or
//! publish the first valid rows of a snapshot whose later rows fail.
use ddonirang_core::ResourceHandle;
use ddonirang_core::platform::{NuriWorld, EntityId, ComponentTag};
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) fn parse_param_overrides(payload:&Value)->Result<std::collections::BTreeMap<String,ddonirang_lang::runtime::Value>,String>{
    use ddonirang_lang::runtime::Value as DdnValue;
    let object=payload.as_object().ok_or("E_WASM_STATE_PARAMS_INVALID: param_overrides는 object여야 합니다")?;
    object.iter().map(|(key,value)|{
        let parsed=match value{
            Value::Bool(v)=>DdnValue::Bool(*v),Value::String(v)=>DdnValue::String(v.clone()),
            Value::Object(_)=>super::json_to_fixed64(value).map(DdnValue::Fixed64)
                .ok_or_else(||format!("E_WASM_STATE_PARAMS_INVALID: param_overrides[{key}] Fixed64 raw_i64 형식 오류"))?,
            Value::Number(_)=>return Err(format!("E_WASM_STATE_PARAMS_INVALID: param_overrides[{key}] binary JSON number는 Fixed64 authority가 아닙니다")),
            _=>return Err(format!("E_WASM_STATE_PARAMS_INVALID: param_overrides[{key}] 지원하지 않는 값")),
        };Ok((key.clone(),parsed))
    }).collect()
}

pub(crate) fn restore_world_resources_snapshot(world:&mut NuriWorld, resources:&Value)->Result<(),String>{
    restore_world_resources_snapshot_with_contracts(world,resources,&[])
}

/// Recorded-contract admission is supplied by the custody owner, never read
/// from a self-asserted JSON flag or replaced by today's registry contents.
pub(crate) fn restore_world_resources_snapshot_with_contracts(world:&mut NuriWorld, resources:&Value, contracts:&[crate::registered_atom_contract::RecordedAtomContract])->Result<(),String>{
    let candidate = parse_resource_candidate(resources, contracts)?;
    *world = candidate.admit(None)?;
    Ok(())
}

pub(crate) fn restore_world_resources_with_tuple_plan(world: &mut NuriWorld, resources: &Value, plan: Option<&crate::ddn_runtime::SavedTupleRestorePlan>) -> Result<(), String> {
    restore_world_resources_with_catalog(world,resources,plan,None)
}
pub(crate) fn restore_world_resources_with_catalog(world:&mut NuriWorld,resources:&Value,plan:Option<&crate::ddn_runtime::SavedTupleRestorePlan>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<(),String>{
    let candidate = parse_resource_candidate_with_catalog(resources,&[],catalog)?;
    *world = candidate.admit(plan)?;
    Ok(())
}

pub(super) fn check_resource_snapshot_structure(resources: &Value) -> Result<(), String> {
    check_resource_snapshot_structure_with_catalog(resources,None)
}
pub(super) fn check_resource_snapshot_structure_with_catalog(resources:&Value,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<(),String>{
    // The staged world cannot escape this byte-only gate. This function grants
    // no source custody and never substitutes for live restore admission.
    drop(parse_resource_candidate_with_catalog(resources,&[],catalog)?);
    Ok(())
}

pub(super) fn check_resource_snapshot_matches_world(resources: &Value, world: &NuriWorld) -> Result<(), String> {
    check_resource_snapshot_matches_world_with_catalog(resources,world,None)
}
pub(super) fn check_resource_snapshot_matches_world_with_catalog(resources:&Value,world:&NuriWorld,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<(),String>{
    let candidate = parse_resource_candidate_with_catalog(resources,&[],catalog)?;
    if candidate.0.state_hash() != world.state_hash() { return Err("E_RESOURCE_SNAPSHOT_INVALID: 실제 재생한 세계와 저장 후보가 다릅니다".into()); }
    Ok(())
}

struct ResourceCandidate(NuriWorld);

/// Join the existing JSON projection with the SAME binary field02/05/06/07.
/// This is a staged value, not Native custody, transaction, or VM authority.
/// No lifetime data or atom contract is accepted from JSON.
pub(crate) fn stage_resource_snapshot_with_complete_frame(
    resources:&Value,frame:&super::complete_state_frame::CompleteStateFrameV1<'_>,
    plan:Option<&crate::ddn_runtime::SavedTupleRestorePlan>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
)->Result<NuriWorld,String>{
    complete_frame_resource_candidate(resources,frame,catalog)?.admit(plan)
}
pub(super) fn check_resource_snapshot_complete_pair(resources:&Value,frame:&super::complete_state_frame::CompleteStateFrameV1<'_>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<(),String>{
    drop(complete_frame_resource_candidate(resources,frame,catalog)?);Ok(())
}
fn complete_frame_resource_candidate(resources:&Value,frame:&super::complete_state_frame::CompleteStateFrameV1<'_>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<ResourceCandidate,String>{
    let invalid=|error|format!("E_VC04_COMPLETE_RESOURCE_BINDING: {error:?}");
    // Joint typed decode checks references against the complete subject table,
    // including forward targets, before the projection is compared.
    drop(frame.decode_value_tables_with_catalog(catalog).map_err(invalid)?);
    let contracts=super::complete_state_tables::decode_registered_atom_evidence(frame.atom_evidence).map_err(invalid)?;
    let custody=super::complete_state_tables::decode_saved_world_custody(frame.subject_custody,frame.pesi).map_err(invalid)?;
    let references=custody.as_ref().map(crate::subject_reference_owner::SubjectReferenceContextV1::new)
        .transpose().map_err(invalid)?;
    let candidate=parse_resource_candidate_with_context(resources,&contracts,references.as_ref(),catalog)?;
    if !candidate.0.component_json_entries().is_empty()||candidate.0.next_entity_cursor()!=0
        ||!candidate.0.resource_handle_entries().is_empty(){
        return Err("E_VC04_COMPLETE_RESOURCE_BINDING: binary 공급이 없는 채널입니다".into());
    }
    let mut values=std::collections::BTreeMap::new();
    for (name,value) in candidate.0.resource_fixed64_entries(){values.insert(name,ddonirang_core::platform::ResourceValue::Fixed64(value));}
    for (name,value) in candidate.0.resource_value_entries(){
        if values.insert(name,value).is_some(){return Err("E_VC04_COMPLETE_RESOURCE_BINDING: 중복 자원입니다".into());}
    }
    let bytes=super::complete_state_tables::encode_resource_table_with_context(&values,catalog,references.as_ref(),&contracts).map_err(invalid)?;
    if bytes!=frame.resources{return Err("E_VC04_COMPLETE_RESOURCE_BINDING: JSON·binary 자원이 다릅니다".into());}
    let mut world=candidate.0;
    if custody.is_some(){world.set_subject_world(ddonirang_core::platform::SubjectWorldStateV1{
        subject_values:frame.subjects.to_vec(),subject_custody:frame.subject_custody.to_vec()});}
    Ok(ResourceCandidate(world))
}

impl ResourceCandidate {
    fn admit(self, plan: Option<&crate::ddn_runtime::SavedTupleRestorePlan>) -> Result<NuriWorld, String> {
        for (tag, value) in self.0.resource_value_entries() {
            if super::tuple_value::requires_source_type(&value) {
                plan.ok_or("E_TUPLE_PERSISTENCE_UNBOUND: 복원 원문·위치씨 공급이 필요합니다")?.admit_resource(&tag, &value)?;
            }
        }
        Ok(self.0)
    }
}

fn parse_resource_candidate(resources:&Value, contracts:&[crate::registered_atom_contract::RecordedAtomContract])->Result<ResourceCandidate,String>{
    parse_resource_candidate_with_catalog(resources,contracts,None)
}
fn parse_resource_candidate_with_catalog(resources:&Value,contracts:&[crate::registered_atom_contract::RecordedAtomContract],catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<ResourceCandidate,String>{
    parse_resource_candidate_with_context(resources,contracts,None,catalog)
}
fn parse_resource_candidate_with_context(resources:&Value,contracts:&[crate::registered_atom_contract::RecordedAtomContract],
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
)->Result<ResourceCandidate,String>{
    crate::registered_atom_contract::verify_contract_set(contracts).map_err(|e|e.code().to_owned())?;
    let invalid=|part:&str|format!("E_RESOURCE_SNAPSHOT_INVALID: {part}");
    let root=resources.as_object().ok_or_else(||invalid("resources는 object여야 합니다"))?;
    if root.get("schema").and_then(Value::as_str)!=Some(super::RESOURCE_SNAPSHOT_SCHEMA_V2){
        return Err("E_RESOURCE_SNAPSHOT_VERSION_MISMATCH: 자원 스냅샷 판이 다릅니다".into());
    }
    if root.get("unit_wire_schema").and_then(Value::as_str)!=Some(super::UNIT_VALUE_WIRE_SCHEMA_V2){
        return Err("E_UNIT_WIRE_CHECKPOINT_MISMATCH: 단위 저장 판이 다릅니다".into());
    }
    // This is a complete checkpoint, not a patch. All existing product
    // channels are mandatory, even when empty; a wrong type is never omission.
    let components=root.get("components").and_then(Value::as_array).ok_or_else(||invalid("components"))?;
    let next_entity=root.get("next_entity").and_then(Value::as_u64).ok_or_else(||invalid("next_entity"))?;
    let map=|key:&str|root.get(key).and_then(Value::as_object).ok_or_else(||invalid(key));
    let json=map("json")?;let fixed=map("fixed64")?;let handles=map("handle")?;let values=map("value_det")?;
    // R1.1: the structural empty lane is retained, never guessed/migrated.
    if !json.is_empty(){return Err(invalid("legacy resource_json 자원은 지원하지 않습니다"));}
    if fixed.keys().any(|name|handles.contains_key(name)||values.contains_key(name))
        || handles.keys().any(|name|values.contains_key(name)){
        return Err(invalid("같은 자원 이름이 여러 저장 통로에 있습니다"));
    }
    let mut candidate=NuriWorld::new();
    let mut seen=BTreeSet::new();
    for row in components{
        let object=row.as_object().ok_or_else(||invalid("component"))?;
        if object.len()!=3{return Err(invalid("component 받는자리"));}
        let entity=row.get("entity").and_then(Value::as_u64).ok_or_else(||invalid("component entity"))?;
        if entity>=next_entity{return Err(invalid("component entity는 저장된 next_entity보다 작아야 합니다"));}
        let tag=row.get("tag").and_then(Value::as_str).ok_or_else(||invalid("component tag"))?;
        let text=row.get("json").and_then(Value::as_str).ok_or_else(||invalid("component json"))?;
        if !seen.insert((entity,tag)){return Err(invalid("component 중복"));}
        candidate.set_component_json(EntityId(entity),ComponentTag(tag.into()),text.into());
    }
    candidate.restore_next_entity_cursor(next_entity);
    if candidate.next_entity_cursor()!=next_entity{return Err(invalid("next_entity 복원 불일치"));}
    for (tag,value) in fixed{
        let parsed=super::json_to_fixed64(value).ok_or_else(||invalid(&format!("fixed64[{tag}]")))?;
        candidate.set_resource_fixed64(tag.clone(),parsed);
    }
    for (tag,value) in handles{
        let raw=match value{
            Value::String(text)=>{
                // Preserve the existing accepted handle projection. Parsing
                // is shared Rust, never an adapter-local identity repair.
                let text=text.trim();let text=text.strip_prefix("handle:").unwrap_or(text);
                let text=text.strip_prefix("자원#").unwrap_or(text);
                u64::from_str_radix(text,16).map_err(|_|invalid(&format!("handle[{tag}]")))?
            },
            Value::Number(number)=>number.as_u64().ok_or_else(||invalid(&format!("handle[{tag}]")))?,
            _=>return Err(invalid(&format!("handle[{tag}]"))),
        };
        candidate.set_resource_handle(tag.clone(),ResourceHandle::from_raw(raw));
    }
    let mut tuple_budget = super::saved_resource_value::ValueDecodeBudget::new();
    for (tag,value) in values{
        candidate.set_resource_value(tag.clone(),super::resource_value_candidate_from_detjson_with_catalog(value,contracts,references,&mut tuple_budget,catalog)?);
    }
    Ok(ResourceCandidate(candidate))
}

#[cfg(test)]
mod tests{
    use super::*;
    use ddonirang_core::Fixed64;
    use serde_json::json;
    #[test]
    fn beta1_atom_world_restore_is_atomic_and_requires_recorded_custody(){
        let (_,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"owner",&[vec![1]]).unwrap();
        let atom=ddonirang_core::platform::ResourceValue::RegisteredAtom(contract.admitted_value(b"owner",&[1]).unwrap().into());
        let mut payload=snapshot();payload["value_det"]=json!({"a":{"type":"bool","value":true},"z":super::super::resource_value_to_detjson(&atom)});
        let mut world=NuriWorld::new();world.set_resource_json("original".into(),"kept".into());
        let before=world.state_hash();
        assert!(restore_world_resources_snapshot(&mut world,&payload).is_err());
        assert_eq!(world.state_hash(),before);
        restore_world_resources_snapshot_with_contracts(&mut world,&payload,&[contract.clone()]).unwrap();
        assert_eq!(world.get_resource_value("z"),Some(atom));
        let restored=world.state_hash();payload["value_det"]["z"]["contract_sha256"]=json!("00".repeat(32));
        assert!(restore_world_resources_snapshot_with_contracts(&mut world,&payload,&[contract]).is_err());
        assert_eq!(world.state_hash(),restored);
    }
    fn snapshot()->Value{
        json!({"schema":super::super::RESOURCE_SNAPSHOT_SCHEMA_V2,"unit_wire_schema":super::super::UNIT_VALUE_WIRE_SCHEMA_V2,
            "components":[],"next_entity":0,"json":{},"fixed64":{"값":{"raw_i64":"-1"}},"handle":{},"value_det":{"설명":{"type":"string","value":"123"}}})
    }
    #[test]
    fn complete_restore_preserves_typed_values_and_replaces_old_world(){
        let mut world=NuriWorld::new();world.set_resource_fixed64("옛값".into(),Fixed64::from_i64(9));
        restore_world_resources_snapshot(&mut world,&snapshot()).unwrap();
        assert_eq!(world.get_resource_fixed64("값"),Some(Fixed64::from_raw_i64(-1)));
        assert_eq!(world.get_resource_value("설명"),Some(ddonirang_core::platform::ResourceValue::String("123".into())));
        assert_eq!(world.get_resource_fixed64("옛값"),None);
    }
    #[test]
    fn clean_break_legacy_and_cross_lane_restore_are_atomic(){
        let mut legacy=snapshot();legacy["json"]=json!({"고정":"참","행동":"왼쪽"});
        let mut duplicate=snapshot();duplicate["value_det"]["값"]=json!({"type":"bool","value":true});
        for payload in [legacy,duplicate]{
            let mut world=NuriWorld::new();world.set_resource_fixed64("보존".into(),Fixed64::from_i64(9));
            let before=format!("{world:?}");
            assert!(restore_world_resources_snapshot(&mut world,&payload).is_err());
            assert_eq!(format!("{world:?}"),before);
        }
        let mut valid=snapshot();valid["value_det"]=json!({"고정":{"type":"bool","value":true},"행동":{"type":"string","value":"참"}});
        let mut world=NuriWorld::new();restore_world_resources_snapshot(&mut world,&valid).unwrap();
        assert_eq!(world.get_resource_value("고정"),Some(ddonirang_core::platform::ResourceValue::Bool(true)));
        assert_eq!(world.get_resource_value("행동"),Some(ddonirang_core::platform::ResourceValue::String("참".into())));
        assert!(world.resource_json_entries().is_empty());
    }
    #[test]
    fn malformed_or_missing_channel_never_silently_drops_saved_values(){
        for key in ["components","next_entity","json","fixed64","handle","value_det"]{
            for absent in [false,true]{
                let mut payload=snapshot();
                if absent{payload.as_object_mut().unwrap().remove(key);}else{payload[key]=Value::Null;}
                let mut world=NuriWorld::new();world.set_resource_fixed64("元".into(),Fixed64::from_i64(7));
                let before=format!("{world:?}");
                assert!(restore_world_resources_snapshot(&mut world,&payload).is_err(),"{key}, absent={absent}");
                assert_eq!(format!("{world:?}"),before);
            }
        }
    }
    #[test]
    fn contradictory_entity_cursor_is_rejected_without_repair_or_publication(){
        for (entity,cursor) in [(0,0),(5,5),(5,2),(u64::MAX,0),(u64::MAX,u64::MAX)]{
            let mut payload=snapshot();payload["next_entity"]=json!(cursor);
            payload["components"]=json!([{"entity":entity,"tag":"위치","json":"{}"}]);
            let mut world=NuriWorld::new();world.set_resource_json("원래".into(),"유지".into());
            let before=format!("{world:?}");
            assert!(restore_world_resources_snapshot(&mut world,&payload).is_err());
            assert_eq!(format!("{world:?}"),before);
        }
        let mut payload=snapshot();payload["next_entity"]=json!(6);
        payload["components"]=json!([{"entity":5,"tag":"위치","json":"{}"}]);
        let mut world=NuriWorld::new();restore_world_resources_snapshot(&mut world,&payload).unwrap();
        assert_eq!(world.next_entity_cursor(),6);assert_eq!(world.spawn().0,6);
    }
    #[test]
    fn late_invalid_row_does_not_publish_earlier_good_rows(){
        for (key,bad) in [("json",json!({"a":"ok","z":17})),("value_det",json!({"a":{"type":"bool","value":true},"z":{"type":"unknown"}}))]{
            let mut payload=snapshot();payload[key]=bad;
            let mut world=NuriWorld::new();world.set_resource_json("original".into(),"kept".into());
            let before=format!("{world:?}");
            assert!(restore_world_resources_snapshot(&mut world,&payload).is_err());
            assert_eq!(format!("{world:?}"),before);
        }
        let mut payload=snapshot();payload["components"]=json!([{ "entity":1,"tag":"位置","json":"{}"},{"entity":1,"tag":"位置","json":"{}"}]);
        assert!(restore_world_resources_snapshot(&mut NuriWorld::new(),&payload).is_err());
    }
}
