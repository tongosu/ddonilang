//! RD-136 common absolute scene cursor. This cache is presentation history,
//! never a world resource, physical clock, or a replacement input tape.
use std::collections::BTreeMap;
use serde_json::{json,Value as Json};
use crate::unit_wire::SnapshotJsonOwner as OwnedJson;

#[derive(Debug,Clone,Default)]
pub struct SceneHistory { frames:BTreeMap<u64,OwnedJson>, encoded_bytes:usize }

/// In-memory ownership only. This is neither a registry identity nor a wire
/// token. Cloning an execution preserves ownership; a separate run cannot
/// insert its frame. No Serialize/Deserialize or JSON reconstruction exists.
#[derive(Debug,Clone)]
pub(crate) struct TableSelectionFrame {
    owner:std::sync::Arc<()>,
    values:crate::bogae_projection::BoundTableSelections,
}
#[derive(Debug,Clone,Default)]
pub(crate) struct TableSelectionTimeline {
    owner:std::sync::Arc<()>,
    source:Option<String>,
    frames:BTreeMap<u64,TableSelectionFrame>,
}
impl TableSelectionTimeline {
    fn check(&self,frame:&TableSelectionFrame)->Result<(),String> {
        if !std::sync::Arc::ptr_eq(&self.owner,&frame.owner) {
            return Err("E_AUTHORED_BOGAE: 다른 실행의 표 선택 이력입니다".into());
        }
        if self.source.as_deref().is_some_and(|s|s!=frame.values.source()) {
            return Err("E_AUTHORED_BOGAE: 표 선택 이력의 원문 판본이 다릅니다".into());
        }
        Ok(())
    }
    pub(crate) fn capture(&self,values:crate::bogae_projection::BoundTableSelections)->Result<TableSelectionFrame,String> {
        let frame=TableSelectionFrame {owner:self.owner.clone(),values};self.check(&frame)?;Ok(frame)
    }
    pub(crate) fn commit(&mut self,madi:u64,frame:TableSelectionFrame)->Result<(),String> {
        self.check(&frame)?;
        if self.frames.get(&madi).is_some_and(|old|old.values!=frame.values) {
            return Err("E_AUTHORED_BOGAE: 확정한 마디의 표 선택을 바꿀 수 없습니다".into());
        }
        self.source=Some(frame.values.source().into());self.frames.insert(madi,frame);Ok(())
    }
    pub(crate) fn resolve_committed(&self,cursor:u64,resolved:&Json)
        ->Result<crate::bogae_projection::BoundTableSelections,String> {
        let frame=self.frames.get(&cursor)
            .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 복원 마디의 typed 표 선택 기록이 없습니다")?;
        self.resolve(cursor,resolved,frame)
    }

    /// Consume the actual SceneHistory result, whose owner identities were
    /// supplied by projection/activation. Never substitute current selectors
    /// for an absent historical frame, or join fields by row/source_ref.
    pub(crate) fn resolve(&self,cursor:u64,resolved:&Json,current:&TableSelectionFrame)
        ->Result<crate::bogae_projection::BoundTableSelections,String> {
        self.check(current)?;
        let mut output=current.values.empty_like();
        let candidates=resolved["candidates"].as_array()
            .ok_or("E_AUTHORED_BOGAE: 해소된 보기 후보가 없습니다")?;
        if candidates.is_empty() {return Err("E_AUTHORED_BOGAE: 해소된 보기 후보가 비어 있습니다".into());}
        for candidate in candidates {
            let artifact=&candidate["artifact"];
            let composition=artifact.get("identity").filter(|id|id.is_object())
                .ok_or("E_AUTHORED_BOGAE: 표 선택의 실제 짜임 정체가 없습니다")?;
            let mut owners=vec![(artifact,None)];
            if let Some(relations)=artifact["relations"].as_array() {
                for relation in relations {
                    let identity=relation.get("identity").filter(|id|id.is_object())
                        .ok_or("E_AUTHORED_BOGAE: 표 선택의 실제 관계 정체가 없습니다")?;
                    owners.push((relation,Some(identity)));
                }
            }
            for (owner,relation) in owners {
                if owner["view"]["kind"]!="result_table" && owner["view"]["kind"]!="scene" {continue;}
                let madi=match owner["resolved_madi"].as_u64() {
                    Some(madi)=>madi,
                    None if owner["time_policy"]["kind"]=="common"=>cursor,
                    _=>return Err("E_BOGAE_TIME_OUT_OF_RANGE: 표 선택의 장면 마디가 해소되지 않았습니다".into()),
                };
                let frame=if madi==cursor {current}else{self.frames.get(&madi)
                    .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디의 typed 표 선택 기록이 없습니다")?};
                self.check(frame)?;
                output.copy_owner_from(&frame.values,composition,relation)?;
            }
        }
        Ok(output)
    }
}

pub(crate) const SPACE3_DURABILITY_UNSUPPORTED: &str =
    "E_PROJECT_RUNTIME_SPACE3_DURABILITY_UNSUPPORTED: 3D 보기는 읽기 전용입니다. 원문 저장과 편집판 실행은 별도입니다";

/// Reject read-only space3 leaves at their actual projection owners before
/// encoding a durable runtime artifact. The display and restore readers stay
/// separate. A historical frame can hold a space3 view after the active one
/// changes.
pub(crate) fn check_durable_scene_projections(runtime: &Json) -> Result<(), &'static str> {
    let frames = &runtime["state"]["snapshot_v2"]["scene_history"]["frames"];
    check_projection_owners(
        runtime.get("bogae_projection"),
        frames.as_array().into_iter().flatten().map(|frame| &frame["projection"]),
    )
}

fn check_projection_owners<'a>(
    current: Option<&'a Json>, history: impl Iterator<Item = &'a Json>,
) -> Result<(), &'static str> {
    fn space3_view(owner: &Json) -> bool {
        let view = &owner["view"];
        view["schema"] == "ddn.scene_space3_projection.v1"
            || (view["kind"] == "scene" && view["spatial_profile"] == "space3")
    }

    fn has_space3(projection: &Json) -> bool {
        projection["candidates"].as_array().into_iter().flatten().any(|candidate| {
            let artifact = &candidate["artifact"];
            space3_view(artifact)
                || artifact["relations"].as_array().into_iter().flatten().any(space3_view)
        })
    }

    if current.into_iter().chain(history).any(has_space3) {
        return Err(SPACE3_DURABILITY_UNSUPPORTED);
    }
    Ok(())
}

fn compact_json(value:&Json)->String {
    let mut bytes=Vec::new();
    crate::unit_wire::write_compact_json(&mut bytes,value).expect("Vec JSON writer");
    String::from_utf8(bytes).expect("JSON UTF-8")
}

fn replace_json(target:&mut Json,value:&Json) {
    let replacement=OwnedJson::clone_from_value(value).into_parent_value();
    drop(OwnedJson::from_value(std::mem::replace(target,replacement)));
}

// Only reject shapes that serde_json's mutable string indexing cannot accept.
// Null/missing objects remain allowed, preserving the existing insertion path.
fn validate_scene_write_shapes(projection:&Json,write_time_lock:bool)->Result<(),String> {
    let object_or_null=|value:&Json|value.is_object()||value.is_null();
    for candidate in projection["candidates"].as_array().into_iter().flatten() {
        if !object_or_null(candidate) || (write_time_lock
            && (!object_or_null(&candidate["artifact"])
                || !object_or_null(&candidate["artifact"]["lock"]))) {
            return Err("E_BOGAE_TIME_POLICY_INVALID: 장면의 쓰기 대상이 객체가 아닙니다".into());
        }
    }
    Ok(())
}

fn requested_madi(policy:&Json,cursor:u64)->Result<u64,String> {
    let number=match policy["kind"].as_str() {
        Some("common")=>cursor as i128,
        Some("fixed")=>policy["madi"].as_u64().ok_or("E_BOGAE_TIME_POLICY_INVALID: 고정 마디가 정수가 아닙니다")? as i128,
        Some("relative")=>cursor as i128 + policy["offset"].as_i64().ok_or("E_BOGAE_TIME_POLICY_INVALID: 상대 마디가 정수가 아닙니다")? as i128,
        _=>return Err("E_BOGAE_TIME_POLICY_INVALID: 지원하지 않는 시간정책입니다".into()),
    };
    u64::try_from(number).map_err(|_|"E_BOGAE_TIME_OUT_OF_RANGE: 지정한 장면 마디가 없습니다".into())
}

fn candidate_at<'a>(projection:&'a Json,identity:&Json)->Result<&'a Json,String> {
    // F5-GA[SCENE_READ_1]: 읽기 전용 serde_json 문자열 인덱스는 형식/키 불일치 때 Null을 반환하며 IndexMut의 삽입 경로를 쓰지 않는다.
    projection["candidates"].as_array().into_iter().flatten()
        .find(|candidate|candidate["artifact"]["identity"]==*identity)
        .ok_or_else(||"E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디에 같은 보개짜임의 장면이 없습니다".into())
}

/// Bind table views to the selected committed prefix, not the host's latest table.
/// Uses the same validated public-observation projector on Native and WASM.
pub fn resolve_result_tables(projection:&mut Json,frames:&[Json],source_identity:&str,revision_identity:&str)->Result<(),String> {
    // Common-time unselected tables also need their own committed projection.
    // SceneHistory legitimately leaves this fast path authored (v1), whereas
    // selected tables already enter through the resolved v2 path below.
    if projection["schema"]=="ddn.bogae_projection.v1" {
        let mut owners:Vec<&Json>=projection["candidates"].as_array().into_iter().flatten()
            .map(|candidate|&candidate["artifact"]).collect();
        let mut has_table=false;
        while let Some(owner)=owners.pop() {
            has_table|=owner["view"]["kind"]=="result_table";
            if let Some(relations)=owner["relations"].as_array() {owners.extend(relations);}
        }
        // Neither manufacture madi zero nor change unrelated scene-only views.
        if !has_table || frames.is_empty() {return Ok(());}
        let cursor=frames.last().and_then(|frame|frame["committed_madi"].as_u64())
            .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 현재 관찰 마디가 없습니다")?;
        validate_scene_write_shapes(projection,true)?;
        let mut resolved=OwnedJson::clone_from_value(projection);
        for candidate in resolved.value_mut()["candidates"].as_array_mut()
            .ok_or("E_BOGAE_TIME_POLICY_INVALID: 장면 후보 목록이 없습니다")? {
            let mut owners=vec![&mut candidate["artifact"]];
            while let Some(owner)=owners.pop() {
                if !owner.is_object() || owner["time_policy"]["kind"]!="common" {
                    return Err("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디 해소가 필요합니다".into());
                }
                owner["resolved_madi"]=json!(cursor);
                if let Some(relations)=owner.get_mut("relations").and_then(Json::as_array_mut) {owners.extend(relations.iter_mut());}
            }
        }
        resolved.value_mut()["schema"]=json!("ddn.bogae_projection.v2");
        resolved.value_mut()["scene_cursor"]=json!(cursor);
        resolve_result_tables(resolved.value_mut(),frames,source_identity,revision_identity)?;
        replace_json(projection,&resolved);
        return Ok(());
    }
    if projection["schema"]!="ddn.bogae_projection.v2" {return Ok(());}
    validate_scene_write_shapes(projection,false)?;
    let mut cache:BTreeMap<u64,OwnedJson>=BTreeMap::new();
    // Validate every requested prefix before changing any owner. A later
    // relation's provenance failure must leave earlier normal views intact.
    let mut pending:Vec<&Json>=projection["candidates"].as_array().into_iter().flatten()
        .map(|candidate|&candidate["artifact"]).collect();
    while let Some(owner)=pending.pop() {
        if owner["view"]["kind"]=="result_table" {
            let madi=owner["resolved_madi"].as_u64().ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 결과표의 장면 마디가 없습니다")?;
            if !cache.contains_key(&madi) {
                let (run_id,bundle)=scene_table_trace(frames,source_identity,revision_identity,madi)?;
                // F5-GA[SCENE_READ_3]: SnapshotJsonOwner의 Deref는 &JsonValue이므로 trace 조회도 읽기 전용 Null 반환 계약을 따른다.
                cache.insert(madi,OwnedJson::from_value(crate::deterministic_debug::build_public_observation_table_projection(&bundle["trace"],&run_id)?));
            }
        }
        if let Some(relations)=owner["relations"].as_array() {pending.extend(relations);}
    }
    let Some(candidates)=projection["candidates"].as_array_mut() else {return Ok(());};
    for candidate in candidates {
        let artifact=&mut candidate["artifact"];
        let mut owners=vec![artifact];
        while let Some(owner)=owners.pop() {
            // F5-GA[SCENE_READ_2]: 비교식의 읽기 전용 JSON 인덱스이며 아래 변경식의 안전성까지 주장하지 않는다.
            if owner["view"]["kind"]=="result_table" {
                let madi=owner["resolved_madi"].as_u64().ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 결과표의 장면 마디가 없습니다")?;
                replace_json(&mut owner["view"]["table_projection"],&cache[&madi]);
            }
            if let Some(relations)=owner.get_mut("relations").and_then(Json::as_array_mut) {owners.extend(relations.iter_mut());}
        }
        candidate["resolved_digest"]=json!(crate::runtime_surface::sha256_hex(
            format!("ddn.bogae_resolved_scene.v1\0{}",compact_json(&candidate["artifact"])).as_bytes()));
    }
    Ok(())
}

fn scene_table_trace(frames:&[Json],source_identity:&str,revision_identity:&str,madi:u64)
    ->Result<(String,OwnedJson),String> {
    if !frames.iter().any(|frame|frame["committed_madi"].as_u64()==Some(madi)) {
        return Err("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디의 관찰 기록이 없습니다".into());
    }
    // The scene-prefix run ID is an existing presentation context. Do not let
    // assigning it erase contradictions in identities actually supplied by
    // the producer. Absent identities still use the existing caller context.
    let mut original_run:Option<&str>=None;
    let mut supplied_lock:Option<&str>=None;
    for frame in frames.iter().filter(|frame|frame["committed_madi"].as_u64().is_some_and(|tick|tick<=madi)) {
        let supplied=|key:&str|->Result<Option<&str>,String> {
            match frame.get(key) {
                None|Some(Json::Null)=>Ok(None),
                Some(Json::String(value))=>Ok((!value.is_empty()).then_some(value.as_str())),
                _=>Err(format!("E_DEBUG_RUN_IDENTITY_MIXED: 공급된 {key} 출처는 문자열이어야 합니다")),
            }
        };
        for (key,expected) in [("source_identity",source_identity),("revision_identity",revision_identity)] {
            if supplied(key)?.is_some_and(|value|value!=expected) {
                return Err(format!("E_DEBUG_RUN_IDENTITY_MIXED: 보기 prefix의 {key} 출처가 다릅니다"));
            }
        }
        for (key,bound) in [("run_id",&mut original_run),("lock_identity",&mut supplied_lock)] {
            if let Some(value)=supplied(key)? {
                if bound.is_some_and(|previous|previous!=value) {
                    return Err(format!("E_DEBUG_RUN_IDENTITY_MIXED: 다른 {key} frame을 한 보기 prefix로 섞을 수 없습니다"));
                }
                *bound=Some(value);
            }
        }
    }
    let run_id=format!("scene:{}",crate::runtime_surface::sha256_hex(format!("{source_identity}\0{revision_identity}\0{madi}").as_bytes()));
    let mut request=OwnedJson::from_value(json!({"project_id":"","run_id":run_id,
        "source_identity":source_identity,"revision_identity":revision_identity,"lock_identity":supplied_lock.unwrap_or(""),"run_status":"completed","frames":[]}));
    for frame in frames.iter().filter(|frame|frame["committed_madi"].as_u64().is_some_and(|tick|tick<=madi)) {
        let mut frame=OwnedJson::clone_from_value(frame);
        frame.value_mut()["run_id"]=json!(run_id);frame.value_mut()["source_identity"]=json!(source_identity);
        frame.value_mut()["revision_identity"]=json!(revision_identity);
        // F5-GA[SCENE_LOCAL_REQUEST]: request의 frames는 이 함수의 빈 배열 생성과 push로만 유지되어 배열 인출이 실패하지 않는다.
        request.value_mut()["frames"].as_array_mut().unwrap().push(frame.into_parent_value());
    }
    let bundle=OwnedJson::from_value(crate::deterministic_debug::build_public_observation_trace(&request)?);
    Ok((run_id,bundle))
}

/// Build into a private candidate: a late owner/run/selection failure cannot
/// partially publish a mixture of full and selected tables.
pub(crate) fn resolve_typed_result_tables(projection:&Json,frames:&[Json],source_identity:&str,
    revision_identity:&str,selections:&crate::bogae_projection::BoundTableSelections)->Result<Json,String> {
    if selections.source()!=source_identity {return Err("E_AUTHORED_BOGAE: 표 선택과 실행 원문이 다릅니다".into());}
    let mut output=OwnedJson::clone_from_value(projection);
    // The existing SceneHistory fast path leaves common-time views authored.
    // Resolve only that current frame here; never use it for fixed/relative views.
    if output["schema"]!="ddn.bogae_projection.v2" {
        let cursor=frames.last().and_then(|frame|frame["committed_madi"].as_u64())
            .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 현재 관찰 마디가 없습니다")?;
        let candidates=output.value_mut()["candidates"].as_array_mut().ok_or("E_AUTHORED_BOGAE: 실제 보기 후보가 없습니다")?;
        for candidate in candidates {
            let mut owners=vec![&mut candidate["artifact"]];
            while let Some(owner)=owners.pop() {
                if owner["time_policy"]["kind"]!="common" {return Err("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디 해소가 필요합니다".into());}
                owner["resolved_madi"]=json!(cursor);
                if let Some(relations)=owner.get_mut("relations").and_then(Json::as_array_mut) {owners.extend(relations.iter_mut());}
            }
        }
        output.value_mut()["schema"]=json!("ddn.bogae_projection.v2");
        output.value_mut()["scene_cursor"]=json!(cursor);
    }
    resolve_result_tables(output.value_mut(),frames,source_identity,revision_identity)?;
    if output["schema"]!="ddn.bogae_projection.v2" {return Err("E_AUTHORED_BOGAE: 해소된 보기 projection이 필요합니다".into());}
    let mut traces:BTreeMap<u64,(String,OwnedJson)>=BTreeMap::new();
    let candidates=output.value_mut()["candidates"].as_array_mut().ok_or("E_AUTHORED_BOGAE: 실제 보기 후보가 없습니다")?;
    for candidate in candidates {
        let composition=OwnedJson::clone_from_value(&candidate["artifact"]["identity"]);
        let mut owners=vec![(&mut candidate["artifact"],None)];
        while let Some((owner,relation))=owners.pop() {
            if owner["view"]["kind"]=="result_table" {
                let selected=selections.checked_selection(&composition,relation.as_ref().map(|id:&OwnedJson|&**id))?;
                if let Some(selected)=selected {
                    let madi=owner["resolved_madi"].as_u64().ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 결과표의 장면 마디가 없습니다")?;
                    if !traces.contains_key(&madi) {traces.insert(madi,scene_table_trace(frames,source_identity,revision_identity,madi)?);}
                    let (run,bundle)=&traces[&madi];
                    let table=OwnedJson::from_value(crate::deterministic_debug::build_selected_public_observation_table_projection(&bundle["trace"],run,selected)?);
                    replace_json(&mut owner["view"]["table_projection"],&table);
                }
            }
            if owner["view"]["kind"]=="scene" {
                if let Some(selected)=selections.checked_xy_selection(&composition,relation.as_ref().map(|id:&OwnedJson|&**id))? {
                    let madi=owner["resolved_madi"].as_u64().ok_or("E_BOGAE_TIME_OUT_OF_RANGE: XY 보기의 마디가 없습니다")?;
                    if !traces.contains_key(&madi) {traces.insert(madi,scene_table_trace(frames,source_identity,revision_identity,madi)?);}
                    let (run,bundle)=&traces[&madi];
                    let (scene,table)=crate::deterministic_debug::build_xy_public_observation_projection(&bundle["trace"],run,selected)?;
                    replace_json(&mut owner["view"],&crate::scene_value::project_view(&ddonirang_lang::runtime::ViewValue::Scene(scene)));
                    replace_json(&mut owner["view"]["table_projection"],&table);
                }
            }
            if let Some(relations)=owner.get_mut("relations").and_then(Json::as_array_mut) {
                for relation in relations {
                    let identity=OwnedJson::clone_from_value(&relation["identity"]);
                    owners.push((relation,Some(identity)));
                }
            }
        }
        candidate["resolved_digest"]=json!(crate::runtime_surface::sha256_hex(
            format!("ddn.bogae_resolved_scene.v1\0{}",compact_json(&candidate["artifact"])).as_bytes()));
    }
    Ok(output.into_parent_value())
}

impl SceneHistory {
    /// Check the actual retained owners without materializing a saved snapshot.
    pub(crate) fn check_durable_projections(&self, current: Option<&Json>) -> Result<(), &'static str> {
        check_projection_owners(current, self.frames.values().map(|projection| &**projection))
    }

    /// Compare retained execution history without recursively cloning a JSON
    /// snapshot. Matching the latest projection alone can lose past views.
    pub(crate) fn same_execution_history(&self,other:&Self)->bool {
        self.encoded_bytes==other.encoded_bytes && self.frames.len()==other.frames.len()
            && self.frames.iter().zip(&other.frames).all(|((tick,left),(other_tick,right))|
                tick==other_tick && left.equals_json(right))
    }

    /// Preview is immutable: any out-of-range relation rejects the whole madi.
    pub fn preview(&self,cursor:u64,current:&Json)->Result<Json,String> {
        // F5-GA[SCENE_READ_4]: 시간정책 탐색의 JSON 조회는 불일치 시 Null이며 아래 정책 검증/변경을 대신하지 않는다.
        let needs_history=current["candidates"].as_array().into_iter().flatten().any(|candidate| {
            let artifact=&candidate["artifact"];
            artifact["time_policy"]["kind"]!="common" || artifact["relations"].as_array().into_iter().flatten()
                .any(|relation|relation["time_policy"]["kind"]!="common")
        });
        if !needs_history {return Ok(OwnedJson::clone_from_value(current).into_parent_value());}
        validate_scene_write_shapes(current,true)?;
        let mut result=OwnedJson::clone_from_value(current);
        let Some(candidates)=result.value_mut()["candidates"].as_array_mut() else {
            return Err("E_BOGAE_TIME_POLICY_INVALID: 장면 후보 목록이 없습니다".into());
        };
        for candidate in candidates {
            let authored=OwnedJson::clone_from_value(&candidate["artifact"]);
            replace_json(&mut candidate["authored_artifact"],&authored);
            candidate["artifact"]["lock"]["time"]=json!("scene_madi.v1");
            // F5-GA[SCENE_READ_5]: 정체와 시간정책의 빌림은 읽기 전용이다. 이 앞뒤의 mutable 경로는 별도 감사 대상으로 남긴다.
            let identity=OwnedJson::clone_from_value(&candidate["artifact"]["identity"]);
            let primary_madi=requested_madi(&candidate["artifact"]["time_policy"],cursor)?;
            let primary_frame=if primary_madi==cursor {current}else{
                self.frames.get(&primary_madi).ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정한 장면 마디가 없습니다")?
            };
            let old=candidate_at(primary_frame,&identity)?;
            replace_json(&mut candidate["artifact"]["view"],&old["artifact"]["view"]);
            candidate["artifact"]["resolved_madi"]=json!(primary_madi);
            if let Some(relations)=candidate["artifact"]["relations"].as_array_mut() {
                for relation in relations {
                    let madi=requested_madi(&relation["time_policy"],cursor)?;
                    let frame=if madi==cursor {current}else{
                        self.frames.get(&madi).ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정한 장면 마디가 없습니다")?
                    };
                    // F5-GA[SCENE_READ_6]: candidate_at의 공유 참조에서 읽으므로 배열 변환 실패는 None이며 인덱스 panic으로 바뀌지 않는다.
                    let prior=candidate_at(frame,&identity)?["artifact"]["relations"].as_array().into_iter().flatten()
                        .find(|old|old["identity"]==relation["identity"])
                        .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디에 같은 작성 관계가 없습니다")?;
                    // Keep current authored policy/identity. Only typed scene
                    // content comes from the explicitly selected scene-madi.
                    for field in ["view","target","target_digest","selection"] {
                        if let Some(value)=prior.get(field) {replace_json(&mut relation[field],value);}
                    }
                    if relation["kind"]=="magnifier" {
                        // The target/view follow this relation's selected time;
                        // picking indices belong to the actually displayed primary.
                        // Reuse exact producer matching, never cross-pose IDs or labels.
                        match crate::bogae_projection::primary_scene_element_selection(
                            &old["artifact"]["view"],&relation["target"]) {
                            Some(selection)=>replace_json(&mut relation["selection"],&selection),
                            None=>{relation.as_object_mut().expect("validated relation object").remove("selection");}
                        }
                    }
                    relation["resolved_madi"]=json!(madi);
                }
            }
            candidate["resolved_digest"]=json!(crate::runtime_surface::sha256_hex(
                format!("ddn.bogae_resolved_scene.v1\0{}",compact_json(&candidate["artifact"])).as_bytes()));
        }
        result.value_mut()["scene_cursor"]=json!(cursor);
        result.value_mut()["schema"]=json!("ddn.bogae_projection.v2");
        Ok(result.into_parent_value())
    }

    /// Caller records only after activation and all madi validation succeed.
    pub fn storage_bytes_after(&self,madi:u64,authored:&Json)->Result<usize,String> {
        let old=self.frames.get(&madi).map(|frame|compact_json(frame).len()).unwrap_or(0);
        self.encoded_bytes.checked_sub(old).and_then(|bytes|bytes.checked_add(compact_json(authored).len()))
            .ok_or_else(||"E_BOGAE_HISTORY_SIZE_OVERFLOW: 장면 이력 크기가 표현 범위를 넘었습니다".into())
    }
    pub fn commit(&mut self,madi:u64,authored:Json) {
        let authored=OwnedJson::from_value(authored);
        self.encoded_bytes=self.storage_bytes_after(madi,&authored).expect("caller preflights result artifact bytes");
        self.frames.insert(madi,authored);
    }
    pub fn clear(&mut self) {self.frames.clear();self.encoded_bytes=0;}

    pub fn restored_projection(&self,cursor:Option<u64>)->Result<Option<Json>,String> {
        let Some(cursor)=cursor else {return Ok(None);};
        let Some(current)=self.frames.get(&cursor) else {return Ok(None);};
        self.preview(cursor,current).map(Some)
    }

    pub fn snapshot(&self)->Json {
        let mut frames=OwnedJson::from_value(json!([]));
        for (madi,projection) in &self.frames {
            let mut row=json!({"madi":madi});
            // F5-GA[SCENE_LOCAL_ROWS]: row는 방금 만든 Object, frames는 이 함수의 Array이며 외부 가변 별칭 없이 삽입/push만 수행한다.
            row["projection"]=projection.clone().into_parent_value();
            frames.value_mut().as_array_mut().unwrap().push(row);
        }
        let digest=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{}",compact_json(&frames)).as_bytes());
        let mut result=json!({"schema":"ddn.bogae_scene_history.v1","sha256":digest});
        // F5-GA[SCENE_LOCAL_RESULT]: result는 바로 앞에서 만든 Object이고 frames 필드 삽입 전 다른 타입으로 바뀌지 않는다.
        result["frames"]=frames.into_parent_value();
        result
    }

    /// Validate separately before the caller replaces live state.
    pub fn from_snapshot(snapshot:Option<&Json>,next_madi:u64)->Result<Self,String> {
        let Some(snapshot)=snapshot else {return Ok(Self::default());};
        let invalid=||"E_BOGAE_HISTORY_SNAPSHOT_INVALID: 장면 이력 저장값이 올바르지 않습니다".to_string();
        if snapshot["schema"]!="ddn.bogae_scene_history.v1" {return Err(invalid());}
        let rows=snapshot["frames"].as_array().ok_or_else(invalid)?;
        let expected=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{}",compact_json(&snapshot["frames"])).as_bytes());
        if snapshot["sha256"].as_str()!=Some(expected.as_str()) {return Err(invalid());}
        let mut history=Self::default();
        let mut previous=None;
        for row in rows {
            let madi=row["madi"].as_u64().ok_or_else(invalid)?;
            if madi>=next_madi || previous.is_some_and(|prior|madi<=prior) {return Err(invalid());}
            let projection=&row["projection"];
            if !projection["candidates"].is_array() {return Err(invalid());}
            let bytes=history.storage_bytes_after(madi,projection)?;
            crate::work_cost::WorkCostMeter::new(Default::default()).set_result_artifact_bytes(bytes)
                .map_err(|error|error.to_string())?;
            history.commit(madi,OwnedJson::clone_from_value(projection).into_parent_value());previous=Some(madi);
        }
        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source_derived_prefix_input()->(Json,Vec<Json>,String,String) {
        let source="채비 { 값:수 := 0. }. (매마디)마다 { 값 <- 값 + 1. (값:값) 내보임. }. 표 := () 표준보기.결과표. 처음 := (보기=표,시간=((번호=0) 표준시간.고정마디)) 표준보개.으뜸. 현재 := (보기=표,자리=#오른쪽) 표준보개.곁. 보기 := (관계들=차림[처음,현재]) 표준보개.짜임.";
        let summary=crate::runtime_surface::run_summary_with_bogae_entry(source,"frame-binding.ddn",Some(3),"v1-core-v25","보기").unwrap();
        let trace=&summary["public_observation_trace"];
        // Test-only reconstruction of the existing debug-frame DTO from real
        // source-produced cells. This is not a new producer transport/wire.
        let frames=trace["samples"].as_array().unwrap().iter().map(|sample| {
            let observations:Vec<_>=sample["cells"].as_array().unwrap().iter().enumerate().map(|(index,cell)| {
                let column=trace["columns"].as_array().unwrap().iter().find(|c|c["identity"]==cell["identity"]).unwrap();
                json!({"committed_madi":sample["committed_madi"],"starts_observation_event":index==0,
                    "name":column["identity"]["name"],"source_ref":column["identity"]["source_ref"],
                    "dtype":column["dtype"],"unit":column["unit"],"format":column["format"],
                    "value_canon":cell["value_canon"],"value_text":cell["value_text"]})
            }).collect();
            json!({"committed_madi":sample["committed_madi"],"state_hash":sample["state_hash"],
                "run_id":trace["run_id"],"source_identity":trace["source_identity"],
                "revision_identity":trace["revision_identity"],"lock_identity":trace["lock_identity"],
                "public_observations":observations})
        }).collect();
        (summary["bogae_projection"].clone(),frames,trace["source_identity"].as_str().unwrap().into(),trace["revision_identity"].as_str().unwrap().into())
    }
    #[test]
    fn time_prefix_rejects_supplied_identity_contradictions_before_changing_views() {
        let (projection,frames,source,revision)=source_derived_prefix_input();
        let mut control=projection.clone();resolve_result_tables(&mut control,&frames,&source,&revision).unwrap();
        let before_frames=frames.clone();
        for key in ["source_identity","revision_identity","run_id","lock_identity"] {
            let mut mixed=frames.clone();
            if key=="lock_identity" {for frame in &mut mixed {frame[key]=json!("VALID_TEST_LOCK");}}
            mixed[2][key]=json!("CONTRADICTORY_TEST_IDENTITY");
            let mut view=projection.clone();
            assert!(resolve_result_tables(&mut view,&mixed,&source,&revision).is_err(),"silently relabelled {key}");
            assert_eq!(view,projection,"late rejection changed an earlier owner for {key}");
        }
        assert_eq!(frames,before_frames);
    }
    #[test]
    fn time_prefix_preserves_supplied_lock_and_legacy_absent_identities() {
        let (projection,mut frames,source,revision)=source_derived_prefix_input();
        for frame in &mut frames {frame["lock_identity"]=json!("VALID_TEST_LOCK");}
        let before=frames.clone();let mut view=projection.clone();
        resolve_result_tables(&mut view,&frames,&source,&revision).unwrap();
        assert_eq!(view["candidates"][0]["artifact"]["view"]["table_projection"]["run"]["lock_identity"],"VALID_TEST_LOCK");
        assert_eq!(frames,before);
        for frame in &mut frames {
            for key in ["source_identity","revision_identity","run_id","lock_identity"] {frame.as_object_mut().unwrap().remove(key);}
        }
        let mut legacy=projection.clone();resolve_result_tables(&mut legacy,&frames,&source,&revision).unwrap();
        assert_eq!(legacy,projection,"normal absent-identity producer input changed");
    }
    #[test]
    fn common_table_binding_keeps_unproduced_and_failed_inputs_unchanged() {
        // Private contract fixtures, not actual source/branch execution proof.
        let mut table=frame(0,json!({"kind":"common"}));
        table["schema"]=json!("ddn.bogae_projection.v1");
        table["candidates"][0]["artifact"]["view"]=json!({"kind":"result_table"});
        let before=table.clone();
        resolve_result_tables(&mut table,&[],"CONTRACT_SOURCE","CONTRACT_REVISION").unwrap();
        assert_eq!(table,before,"no recorded madi must not invent an empty table or cursor");
        let frames=[json!({"committed_madi":0,"state_hash":"CONTRACT_STATE","public_observations":[]})];
        let mut scene=frame(0,json!({"kind":"common"}));scene["schema"]=json!("ddn.bogae_projection.v1");
        let original_scene=scene.clone();
        resolve_result_tables(&mut scene,&frames,"CONTRACT_SOURCE","CONTRACT_REVISION").unwrap();
        assert_eq!(scene,original_scene,"scene-only common fast path remains unchanged");
        table["candidates"][0]["artifact"]["relations"][0]["time_policy"]=json!({"kind":"fixed","madi":0});
        let malformed=table.clone();
        assert!(resolve_result_tables(&mut table,&frames,"CONTRACT_SOURCE","CONTRACT_REVISION").is_err());
        assert_eq!(table,malformed,"late policy failure must not publish partial resolution");
    }
    #[test]
    fn rd694_scene_history_owns_deep_saved_json_and_late_rejection() {
        std::thread::Builder::new().stack_size(128*1024).spawn(|| {
            let mut payload=Json::Null;
            for _ in 0..20_000 {payload=Json::Array(vec![payload]);}
            let mut authored=OwnedJson::from_value(frame(10,json!({"kind":"common"})));
            authored.value_mut()["candidates"][0]["artifact"]["view"]["payload"]=payload;
            let mut history=SceneHistory::default();
            let bytes=history.storage_bytes_after(0,&authored).unwrap();
            history.commit(0,authored.clone().into_parent_value());
            assert_eq!(history.encoded_bytes,bytes);
            let copy=history.clone();
            assert!(history.same_execution_history(&copy));
            let saved=OwnedJson::from_value(history.snapshot());
            let restored=SceneHistory::from_snapshot(Some(&saved),1).unwrap();
            assert!(history.same_execution_history(&restored));
            let preview=OwnedJson::from_value(restored.preview(0,&authored).unwrap());
            assert!(preview.equals_json(&authored));
            let mut fixed=authored.clone();
            fixed.value_mut()["candidates"][0]["artifact"]["time_policy"]=json!({"kind":"fixed","madi":0});
            let resolved=OwnedJson::from_value(restored.preview(1,&fixed).unwrap());
            assert_eq!(resolved["candidates"][0]["artifact"]["resolved_madi"],0);
            fixed.value_mut()["candidates"][0]["artifact"]["time_policy"]["madi"]=json!(99);
            assert!(restored.preview(1,&fixed).is_err());
            // The first row has been admitted into a private history before
            // this second row rejects; its deep JSON must still clean up.
            let mut bad=saved.clone();
            bad.value_mut()["frames"].as_array_mut().unwrap().push(json!({"madi":1,"projection":{"candidates":[]}}));
            let digest=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{}",compact_json(&bad["frames"])).as_bytes());
            bad.value_mut()["sha256"]=json!(digest);
            assert!(SceneHistory::from_snapshot(Some(&bad),1).is_err());
            assert!(history.same_execution_history(&restored));
            history.commit(0,authored.into_parent_value());
            assert_eq!(history.encoded_bytes,bytes);
            history.clear(); assert_eq!(history.encoded_bytes,0);
        }).unwrap().join().unwrap();
    }
    #[test]
    fn a3_rd136_actual_native_table_uses_selected_committed_prefix() {
        // A3-TIME-P1: no harness rewrite; this is the public V25 source frontdoor.
        let source="채비 { 값:수 := 0. }. (매마디)마다 { 값 <- 값 + 1. (값:값) 내보임. }. 보기 := () 표준보기.결과표. 현재 := (보기=보기) 표준보개.으뜸. 처음 := (보기=보기,자리=#오른쪽,시간=((번호=0) 표준시간.고정마디)) 표준보개.곁. 짜임 := (관계들=차림[현재,처음]) 표준보개.짜임.";
        let summary=crate::runtime_surface::run_summary_with_bogae_entry(source,"table-time.ddn",Some(3),"v1-core-v25","짜임").unwrap();
        let projection=&summary["bogae_projection"];
        let relations=&projection["candidates"][0]["artifact"]["relations"];
        assert_eq!(relations[1]["resolved_madi"],0,"{summary}");
        let past=&relations[1]["view"]["table_projection"]["tables"][0]["rows"];
        assert_eq!(past.as_array().expect("historical observation table").len(),1);
        assert_eq!(past[0]["cells"][0]["value_text"],"1");
        let now=&relations[0]["view"]["table_projection"]["tables"][0]["rows"];
        assert_eq!(now.as_array().unwrap().len(),3);
        assert_eq!(now[2]["cells"][0]["value_text"],"3");
        let reordered=source.replace("보기=보기,자리=#오른쪽,시간=((번호=0) 표준시간.고정마디)",
            "시간=((번호=0) 표준시간.고정마디),자리=#오른쪽,보기=보기");
        let reordered=crate::runtime_surface::run_summary_with_bogae_entry(&reordered,"table-time.ddn",Some(3),"v1-core-v25","짜임").unwrap();
        assert_eq!(reordered["bogae_projection"]["candidates"][0]["artifact"]["relations"][1]["view"]["table_projection"]["tables"][0]["rows"],*past);
    }
    #[test]
    fn a3_rd136_history_roundtrip_and_corruption_rejection() {
        let mut history=SceneHistory::default();
        history.commit(0,frame(10,json!({"kind":"common"})));
        let saved=history.snapshot();
        let legacy_rows=json!([{"madi":0,"projection":frame(10,json!({"kind":"common"}))}]);
        let legacy_digest=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{legacy_rows}").as_bytes());
        assert_eq!(saved,json!({"schema":"ddn.bogae_scene_history.v1","frames":legacy_rows,"sha256":legacy_digest}));
        assert!(history.restored_projection(Some(1)).unwrap().is_none(),"restoration never substitutes the last available frame for an absent current frame");
        let restored=SceneHistory::from_snapshot(Some(&saved),1).unwrap();
        assert_eq!(restored.snapshot(),saved);
        assert!(SceneHistory::from_snapshot(Some(&saved),0).is_err());
        let mut corrupt=saved.clone();corrupt["frames"][0]["projection"]["candidates"][0]["artifact"]["view"]["x"]=json!(99);
        assert!(SceneHistory::from_snapshot(Some(&corrupt),1).is_err());
        assert_eq!(history.snapshot(),saved);
        use ddonirang_lang::runtime::BogaeTimePolicy as Policy;
        let policies=[Policy::Common,Policy::Fixed(0),Policy::Fixed(1),Policy::Relative(0),Policy::Relative(-1)];
        let texts:std::collections::BTreeSet<_>=policies.into_iter().map(Policy::canonical_text).collect();
        assert_eq!(texts.len(),5,"different typed policies cannot collide as canonical keys");
    }
    fn frame(x:u64,policy:Json)->Json {
        json!({"candidates":[{"artifact":{"identity":{"module":"exact-source","composition_declaration":[0,20]},
            "view":{"kind":"scene","x":x},"time_policy":{"kind":"common"},
            "relations":[{"identity":{"occurrence":0},"kind":"side","view":{"kind":"scene","x":x},"time_policy":policy}]}}]})
    }
    #[test]
    fn a3_rd136_fixed_relative_and_missing_are_exact_not_clamped() {
        let mut history=SceneHistory::default();history.commit(0,frame(10,json!({"kind":"common"})));
        history.commit(1,frame(20,json!({"kind":"common"})));
        let current=frame(30,json!({"kind":"relative","offset":-1}));
        assert_eq!(history.preview(2,&current).unwrap()["candidates"][0]["artifact"]["relations"][0]["view"]["x"],20);
        let fixed=frame(30,json!({"kind":"fixed","madi":0}));
        assert_eq!(history.preview(2,&fixed).unwrap()["candidates"][0]["artifact"]["relations"][0]["view"]["x"],10);
        for policy in [json!({"kind":"fixed","madi":9}),json!({"kind":"relative","offset":-3})] {
            assert!(history.preview(2,&frame(30,policy)).unwrap_err().contains("E_BOGAE_TIME_OUT_OF_RANGE"));
        }
        assert_eq!(history.frames.len(),2,"a rejected preview cannot advance scene history");
        assert_eq!(history.preview(2,&current).unwrap()["scene_cursor"],2);
    }
    #[test]
    fn a3_rd136_actual_source_uses_same_history_at_the_runtime_frontdoor() {
        use crate::ddn_runtime::{DdnProgram,DdnRunner};
        use ddonirang_core::platform::{DetNuri,Nuri};
        use ddonirang_core::{Fixed64,InputSnapshot,InputSource};
        let source=include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_scene_time.ddn");
        let program=DdnProgram::from_v25_source(source,"scene-time.ddn").unwrap();
        let mut runner=DdnRunner::new(program,"매마디").with_bogae_activation();
        let mut world=DetNuri::new();
        for madi in 0..3 {
            let input=InputSnapshot {tick_id:madi,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
                pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
            let output=runner.run_update(world.world(),&input,&Default::default()).unwrap();
            let projection=output.bogae_projection.as_ref().unwrap();
            let relations=&projection["candidates"][0]["artifact"]["relations"];
            assert_eq!(relations[0]["view"]["elements"][0]["center"][0],Fixed64::from_i64(madi as i64+1).raw_i64().to_string());
            assert_eq!(relations[1]["view"]["elements"][0]["center"][0],Fixed64::ONE.raw_i64().to_string());
            assert_eq!(relations[1]["resolved_madi"],0);
            world.apply_patch(&output.patch,madi,&mut ddonirang_core::signals::VecSignalSink::default());
        }
    }
    #[test]
    fn time_selection_matches_displayed_primary_in_both_time_directions() {
        use crate::ddn_runtime::{DdnProgram,DdnRunner};
        use ddonirang_core::platform::{DetNuri,Nuri};
        use ddonirang_core::{Fixed64,InputSnapshot,InputSource};
        for (source,expected,primary_madi,magnifier_madi) in [
            (include_str!("../tests/bogae_xy/기본과돋_서로다른시각_A.또니"),0,0,1),
            (include_str!("../tests/bogae_xy/기본과돋_서로다른시각_B.또니"),1,1,0),
        ] {
            let program=DdnProgram::from_v25_source(source,"scene-time-selection.ddn").unwrap();
            let mut runner=DdnRunner::new(program,"매마디").with_bogae_activation();
            let mut world=DetNuri::new();
            for madi in 0..2 {
                let input=InputSnapshot {tick_id:madi,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
                    pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
                let output=runner.run_update(world.world(),&input,&Default::default()).unwrap();
                if madi==1 {
                    let artifact=&output.bogae_projection.as_ref().unwrap()["candidates"][0]["artifact"];
                    let relation=artifact["relations"].as_array().unwrap().iter().find(|r|r["kind"]=="magnifier").unwrap();
                    assert_eq!(artifact["resolved_madi"],primary_madi);
                    assert_eq!(relation["resolved_madi"],magnifier_madi);
                    assert_eq!(relation["selection"]["element_indices"],json!([expected]));
                    assert_eq!(artifact["view"]["elements"][expected],relation["target"]["element"]);
                    assert_eq!(relation["view"]["elements"][0],relation["target"]["element"]);
                }
                world.apply_patch(&output.patch,madi,&mut ddonirang_core::signals::VecSignalSink::default());
            }
        }
    }
    #[test]
    fn time_selection_matching_does_not_create_targets_or_persistent_identity() {
        let circle=json!({"kind":"circle","center":["0","0"],"radius":"1"});
        let target=json!({"kind":"scene_element","element":circle});
        let primary=json!({"kind":"scene","elements":[circle.clone(),circle.clone()]});
        let before=(primary.clone(),target.clone());
        assert_eq!(crate::bogae_projection::primary_scene_element_selection(&primary,&target),
            Some(json!({"kind":"primary_scene_element.v1","element_indices":[0,1]})));
        let other=json!({"kind":"scene","elements":[]});
        assert_eq!(crate::bogae_projection::primary_scene_element_selection(&other,&target),
            Some(json!({"kind":"primary_scene_element.v1","element_indices":[]})));
        assert_eq!(crate::bogae_projection::primary_scene_element_selection(&primary,&json!({"kind":"scalar"})),None);
        assert_eq!((primary,target),before);
    }

}
