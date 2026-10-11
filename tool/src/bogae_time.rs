//! RD-136 common absolute scene cursor. This cache is presentation history,
//! never a world resource, physical clock, or a replacement input tape.
use std::collections::BTreeMap;
use serde_json::{json,Value as Json};

#[derive(Debug,Clone,Default)]
pub struct SceneHistory { frames:BTreeMap<u64,Json>, encoded_bytes:usize }

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
    projection["candidates"].as_array().into_iter().flatten()
        .find(|candidate|candidate["artifact"]["identity"]==*identity)
        .ok_or_else(||"E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디에 같은 보개짜임의 장면이 없습니다".into())
}

/// Bind table views to the selected committed prefix, not the host's latest table.
/// Uses the same validated public-observation projector on Native and WASM.
pub fn resolve_result_tables(projection:&mut Json,frames:&[Json],source_identity:&str,revision_identity:&str)->Result<(),String> {
    if projection["schema"]!="ddn.bogae_projection.v2" {return Ok(());}
    let mut cache:BTreeMap<u64,Json>=BTreeMap::new();
    let Some(candidates)=projection["candidates"].as_array_mut() else {return Ok(());};
    for candidate in candidates {
        let artifact=&mut candidate["artifact"];
        let mut owners=vec![artifact];
        while let Some(owner)=owners.pop() {
            if owner["view"]["kind"]=="result_table" {
                let madi=owner["resolved_madi"].as_u64().ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 결과표의 장면 마디가 없습니다")?;
                if !cache.contains_key(&madi) {
                    if !frames.iter().any(|frame|frame["committed_madi"].as_u64()==Some(madi)) {
                        return Err("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디의 관찰 기록이 없습니다".into());
                    }
                    let run_id=format!("scene:{}",crate::runtime_surface::sha256_hex(format!("{source_identity}\0{revision_identity}\0{madi}").as_bytes()));
                    let prefix:Vec<Json>=frames.iter().filter(|frame|frame["committed_madi"].as_u64().is_some_and(|tick|tick<=madi)).map(|frame| {
                        let mut frame=frame.clone();
                        frame["run_id"]=json!(run_id);frame["source_identity"]=json!(source_identity);frame["revision_identity"]=json!(revision_identity);frame
                    }).collect();
                    let bundle=crate::deterministic_debug::build_public_observation_trace(&json!({"project_id":"","run_id":run_id,
                        "source_identity":source_identity,"revision_identity":revision_identity,"run_status":"completed","frames":prefix}))?;
                    cache.insert(madi,crate::deterministic_debug::build_public_observation_table_projection(&bundle["trace"],&run_id)?);
                }
                owner["view"]["table_projection"]=cache[&madi].clone();
            }
            if let Some(relations)=owner.get_mut("relations").and_then(Json::as_array_mut) {owners.extend(relations.iter_mut());}
        }
        candidate["resolved_digest"]=json!(crate::runtime_surface::sha256_hex(
            format!("ddn.bogae_resolved_scene.v1\0{}",candidate["artifact"]).as_bytes()));
    }
    Ok(())
}

impl SceneHistory {
    /// Preview is immutable: any out-of-range relation rejects the whole madi.
    pub fn preview(&self,cursor:u64,current:&Json)->Result<Json,String> {
        let needs_history=current["candidates"].as_array().into_iter().flatten().any(|candidate| {
            let artifact=&candidate["artifact"];
            artifact["time_policy"]["kind"]!="common" || artifact["relations"].as_array().into_iter().flatten()
                .any(|relation|relation["time_policy"]["kind"]!="common")
        });
        if !needs_history {return Ok(current.clone());}
        let mut result=current.clone();
        let Some(candidates)=result["candidates"].as_array_mut() else {
            return Err("E_BOGAE_TIME_POLICY_INVALID: 장면 후보 목록이 없습니다".into());
        };
        for candidate in candidates {
            candidate["authored_artifact"]=candidate["artifact"].clone();
            candidate["artifact"]["lock"]["time"]=json!("scene_madi.v1");
            let identity=candidate["artifact"]["identity"].clone();
            let primary_madi=requested_madi(&candidate["artifact"]["time_policy"],cursor)?;
            let primary_frame=if primary_madi==cursor {current}else{
                self.frames.get(&primary_madi).ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정한 장면 마디가 없습니다")?
            };
            let old=candidate_at(primary_frame,&identity)?;
            candidate["artifact"]["view"]=old["artifact"]["view"].clone();
            candidate["artifact"]["resolved_madi"]=json!(primary_madi);
            if let Some(relations)=candidate["artifact"]["relations"].as_array_mut() {
                for relation in relations {
                    let madi=requested_madi(&relation["time_policy"],cursor)?;
                    let frame=if madi==cursor {current}else{
                        self.frames.get(&madi).ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정한 장면 마디가 없습니다")?
                    };
                    let prior=candidate_at(frame,&identity)?["artifact"]["relations"].as_array().into_iter().flatten()
                        .find(|old|old["identity"]==relation["identity"])
                        .ok_or("E_BOGAE_TIME_OUT_OF_RANGE: 지정 마디에 같은 작성 관계가 없습니다")?;
                    // Keep current authored policy/identity. Only typed scene
                    // content comes from the explicitly selected scene-madi.
                    for field in ["view","target","target_digest","selection"] {
                        if let Some(value)=prior.get(field) {relation[field]=value.clone();}
                    }
                    relation["resolved_madi"]=json!(madi);
                }
            }
            candidate["resolved_digest"]=json!(crate::runtime_surface::sha256_hex(
                format!("ddn.bogae_resolved_scene.v1\0{}",candidate["artifact"]).as_bytes()));
        }
        result["scene_cursor"]=json!(cursor);
        result["schema"]=json!("ddn.bogae_projection.v2");
        Ok(result)
    }

    /// Caller records only after activation and all madi validation succeed.
    pub fn storage_bytes_after(&self,madi:u64,authored:&Json)->Result<usize,String> {
        let old=self.frames.get(&madi).map(|frame|frame.to_string().len()).unwrap_or(0);
        self.encoded_bytes.checked_sub(old).and_then(|bytes|bytes.checked_add(authored.to_string().len()))
            .ok_or_else(||"E_BOGAE_HISTORY_SIZE_OVERFLOW: 장면 이력 크기가 표현 범위를 넘었습니다".into())
    }
    pub fn commit(&mut self,madi:u64,authored:Json) {
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
        let frames:Vec<Json>=self.frames.iter().map(|(madi,projection)|json!({"madi":madi,"projection":projection})).collect();
        let digest=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{}",json!(frames)).as_bytes());
        json!({"schema":"ddn.bogae_scene_history.v1","frames":frames,"sha256":digest})
    }

    /// Validate separately before the caller replaces live state.
    pub fn from_snapshot(snapshot:Option<&Json>,next_madi:u64)->Result<Self,String> {
        let Some(snapshot)=snapshot else {return Ok(Self::default());};
        let invalid=||"E_BOGAE_HISTORY_SNAPSHOT_INVALID: 장면 이력 저장값이 올바르지 않습니다".to_string();
        if snapshot["schema"]!="ddn.bogae_scene_history.v1" {return Err(invalid());}
        let rows=snapshot["frames"].as_array().ok_or_else(invalid)?;
        let expected=crate::runtime_surface::sha256_hex(format!("ddn.bogae_scene_history.v1\0{}",json!(rows)).as_bytes());
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
            history.commit(madi,projection.clone());previous=Some(madi);
        }
        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a3_rd136_actual_native_table_uses_selected_committed_prefix() {
        // A3-TIME-P1: no harness rewrite; this is the public V25 source frontdoor.
        let source="채비 { 값:수 := 0. }. (매마디)마다 { 값 <- 값 + 1. (값:값) 보임. }. 보기 := () 표준보기.결과표. 현재 := (보기=보기) 표준보개.으뜸. 처음 := (보기=보기,자리=#오른쪽,시간=((번호=0) 표준시간.고정마디)) 표준보개.곁. 짜임 := (관계들=차림[현재,처음]) 표준보개.짜임.";
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
}
