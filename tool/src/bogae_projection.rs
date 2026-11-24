//! Non-world projection of authored typed values. This is not a replay format
//! or a persistent identity: source + declaration owns temporary execution IDs.
use ddonirang_lang::runtime::{BogaeRelationValue, Value};
use ddonirang_lang::{CanonProgram, ExprKind, Stmt};
use serde_json::{json, Value as Json};
use std::collections::HashMap;

/// Explicit non-world observation constructors. A named project entry advances
/// the world before these authored root views are materialized (RD-137/138).
/// Numeric definitions and arbitrary user calls are not reordered.
pub(crate) fn is_observation_definition(statement: &Stmt) -> bool {
    let Stmt::Definition { value, .. } = statement else { return false; };
    let ExprKind::Call { func, .. } = &value.kind else { return false; };
    is_observation_constructor(func)
}

fn is_observation_constructor(func: &str) -> bool {
    matches!(func, "표준보기.장면" | "표준보기.결과표" | "표준보개.으뜸"
        | "표준보개.짜임" | "표준보개.겹" | "표준보개.곁" | "표준보개.돋")
}

pub fn project_time(policy: ddonirang_lang::runtime::BogaeTimePolicy) -> Json {
    use ddonirang_lang::runtime::BogaeTimePolicy;
    match policy { BogaeTimePolicy::Common=>json!({"kind":"common"}),
        BogaeTimePolicy::Fixed(madi)=>json!({"kind":"fixed","madi":madi}),
        BogaeTimePolicy::Relative(offset)=>json!({"kind":"relative","offset":offset}) }
}

pub fn attach(state: &mut Json, projection: Option<&Json>) {
    if let (Some(object), Some(projection)) = (state.as_object_mut(), projection) {
        object.insert("bogae_projection".into(), projection.clone());
    }
}

/// An explicitly bound scene object has declaration identity, not pose identity.
/// This is run-local; it creates no persistent/world entity or guessed alias.
fn authored_object_identity(program: &CanonProgram, source: &str,
    resources: &HashMap<String,Value>, relation: &BogaeRelationValue) -> Option<Json> {
    for stmt in &program.root_stmts {
        let Stmt::Definition {name,value,..}=stmt else {continue;};
        let Some(Value::BogaeRelation(bound))=resources.get(name) else {continue;};
        if bound.source_location()!=relation.source_location() {continue;}
        let ExprKind::Call {func,args}=&value.kind else {continue;};
        if func!="표준보개.돋" {continue;}
        let arg=args.iter().find(|arg|arg.resolved_pin.as_deref()==Some("대상")).or(args.first())?;
        let ExprKind::Var(object_name)=&arg.expr.kind else {return None;};
        if !matches!(resources.get(object_name),Some(Value::SceneElement(_))) {return None;}
        let span=program.root_stmts.iter().find_map(|stmt|match stmt {
            Stmt::Definition {name,span,..} if name==object_name=>Some(span), _=>None
        })?;
        let identity=json!({"scope":"temporary_execution_object","module":source,
            "object_declaration":[span.start,span.end]});
        let id=crate::runtime_surface::sha256_hex(format!("ddn.bogae_object.v1\0{identity}").as_bytes());
        return Some(json!({"id":id,"identity":identity,"label":object_name}));
    }
    None
}

/// Existing exact typed element-value matching, scoped to the displayed primary
/// view. Indices are local display coordinates, never persistent object IDs.
pub(crate) fn primary_scene_element_selection(view:&Json,target:&Json)->Option<Json> {
    if target["kind"]!="scene_element" {return None;}
    let indices:Vec<_>=view["elements"].as_array().into_iter().flatten().enumerate()
        .filter(|(_,element)|**element==target["element"])
        .map(|(index,_)|index).collect();
    Some(json!({"kind":"primary_scene_element.v1","element_indices":indices}))
}

/// Activation, unlike pure value construction, must fail before world commit.
pub fn admit_activation(projection: Option<&Json>) -> Result<(), String> {
    let Some(projection) = projection else { return Ok(()); };
    if projection["status"] == "ready" { return Ok(()); }
    let diagnostic = &projection["diagnostic"];
    Err(format!("{}: {}", diagnostic["code"].as_str().unwrap_or("E_BOGAE_PROJECTION_INVALID"),
        diagnostic["message"].as_str().unwrap_or("보개 활성화 요청을 해소할 수 없습니다")))
}

pub fn resolve(
    program: &CanonProgram,
    source_identity: &str,
    resources: &HashMap<String, Value>,
) -> Option<Json> {
    resolve_selected(program, source_identity, resources, None)
}

/// Private typed companion created from the same source/resources as the JSON
/// projection. Assembly positions are temporary coordinates, never identities.
#[derive(Debug,Clone,PartialEq,Eq)]
enum ObservationViewSelection { Table(ddonirang_lang::runtime::ResultTableSelection), Xy(ddonirang_lang::runtime::XySelection) }
pub(crate) struct TypedTableProjection {
    projection: Option<Json>,
    selections: Vec<(usize,Option<usize>,ObservationViewSelection)>,
    source: String,
}
#[derive(Debug,Clone,Default,PartialEq,Eq)]
pub(crate) struct BoundTableSelections {
    source: String,
    owners: Vec<(String,Option<String>)>,
    rows: Vec<(String,Option<String>,ObservationViewSelection)>,
}
impl BoundTableSelections {
    pub(crate) fn is_empty(&self)->bool { self.rows.is_empty() }
    pub(crate) fn source(&self)->&str { &self.source }
    pub(crate) fn empty_like(&self)->Self {Self {source:self.source.clone(),owners:Vec::new(),rows:Vec::new()}}
    pub(crate) fn copy_owner_from(&mut self,other:&Self,composition:&Json,relation:Option<&Json>)->Result<(),String> {
        if self.source!=other.source {return Err("E_AUTHORED_BOGAE: 서로 다른 원문의 표 선택 이력입니다".into());}
        let key=(composition.to_string(),relation.map(Json::to_string));
        if !other.owners.contains(&key) {return Err("E_AUTHORED_BOGAE: 해당 마디의 실제 보기 소유자가 아닙니다".into());}
        if self.owners.contains(&key) {return Err("E_AUTHORED_BOGAE: 해소된 보기 소유자가 중복됐습니다".into());}
        self.owners.push(key.clone());
        if let Some((_,_,selection))=other.rows.iter().find(|(c,r,_)|c==&key.0 && r==&key.1) {
            if self.rows.iter().any(|(c,r,_)|*c==key.0 && *r==key.1) {
                return Err("E_AUTHORED_BOGAE: 해소된 보기 소유자가 중복됐습니다".into());
            }
            self.rows.push((key.0,key.1,selection.clone()));
        }
        Ok(())
    }
    pub(crate) fn checked_selection(&self,composition:&Json,relation:Option<&Json>)
        ->Result<Option<&ddonirang_lang::runtime::ResultTableSelection>,String> {
        let key=(composition.to_string(),relation.map(Json::to_string));
        if !self.owners.contains(&key) {return Err("E_AUTHORED_BOGAE: 표 선택의 실제 보기 소유자가 없습니다".into());}
        Ok(self.selection(composition,relation))
    }
    pub(crate) fn checked_xy_selection(&self,composition:&Json,relation:Option<&Json>)
        ->Result<Option<&ddonirang_lang::runtime::XySelection>,String> {
        let key=(composition.to_string(),relation.map(Json::to_string));
        if !self.owners.contains(&key) {return Err("E_AUTHORED_BOGAE: XY의 실제 보기 소유자가 없습니다".into());}
        Ok(self.rows.iter().find(|(c,r,_)|c==&key.0 && r==&key.1).and_then(|(_,_,s)|match s {ObservationViewSelection::Xy(s)=>Some(s),_=>None}))
    }
    pub(crate) fn selection(&self,composition:&Json,relation:Option<&Json>)
        ->Option<&ddonirang_lang::runtime::ResultTableSelection> {
        let composition=composition.to_string();let relation=relation.map(Json::to_string);
        self.rows.iter().find(|(c,r,_)|*c==composition && *r==relation).and_then(|(_,_,s)|match s {ObservationViewSelection::Table(s)=>Some(s),_=>None})
    }
}
impl TypedTableProjection {
    pub(crate) fn has_projection(&self)->bool {self.projection.is_some()}
    /// The existing activation, not a caller's identity string, supplies the
    /// persistent owners. No raw JSON constructor or deserialize is exposed.
    pub(crate) fn activate(self,activation:Option<&crate::authored_bogae::Activation>)
        ->Result<(Option<Json>,BoundTableSelections),String> {
        let projection=match activation {Some(a)=>a.apply(self.projection)?,None=>self.projection};
        let mut bound=BoundTableSelections {source:self.source,owners:Vec::new(),rows:Vec::new()};
        // Preserve positive ownership even for an unselected view: absence of
        // a selected row must never stand in for a missing/mixed owner.
        for candidate in projection.as_ref().and_then(|p|p["candidates"].as_array()).into_iter().flatten() {
            let artifact=&candidate["artifact"];
            let composition=artifact.get("identity").filter(|id|id.is_object())
                .ok_or("E_AUTHORED_BOGAE: 실제 보기의 짜임 정체가 없습니다")?.to_string();
            let mut keys=vec![(composition.clone(),None)];
            for relation in artifact["relations"].as_array().into_iter().flatten() {
                let identity=relation.get("identity").filter(|id|id.is_object())
                    .ok_or("E_AUTHORED_BOGAE: 실제 보기의 관계 정체가 없습니다")?.to_string();
                keys.push((composition.clone(),Some(identity)));
            }
            for key in keys {
                if bound.owners.contains(&key) {return Err("E_AUTHORED_BOGAE: 실제 보기 소유자가 중복됐습니다".into());}
                bound.owners.push(key);
            }
        }
        for (candidate,relation,selection) in self.selections {
            let artifact=projection.as_ref().and_then(|p|p["candidates"].get(candidate))
                .and_then(|c|c.get("artifact")).ok_or("E_AUTHORED_BOGAE: 열 선택의 실제 보기 소유자가 없습니다")?;
            let composition=artifact.get("identity").filter(|id|id.is_object())
                .ok_or("E_AUTHORED_BOGAE: 열 선택의 짜임 정체가 없습니다")?.to_string();
            let owner=match relation {Some(i)=>artifact["relations"].get(i)
                .ok_or("E_AUTHORED_BOGAE: 열 선택의 실제 관계가 없습니다")?,None=>artifact};
            let expected=match &selection {ObservationViewSelection::Table(_)=>"result_table",ObservationViewSelection::Xy(_)=>"scene"};
            if owner["view"]["kind"]!=expected {return Err("E_AUTHORED_BOGAE: 선택과 보기 종류가 다릅니다".into());}
            let relation=relation.map(|_|owner["identity"].to_string());
            if bound.rows.iter().any(|(c,r,_)|*c==composition && *r==relation) {
                return Err("E_AUTHORED_BOGAE: 같은 보기 소유자의 열 선택이 중복됐습니다".into());
            }
            bound.rows.push((composition,relation,selection));
        }
        Ok((projection,bound))
    }
}

pub(crate) fn resolve_selected_with_tables(program:&CanonProgram,source_identity:&str,
    resources:&HashMap<String,Value>,entry:Option<&str>)->Result<TypedTableProjection,String> {
    let projection=resolve_selected_raw(program,source_identity,resources,entry);
    let has_selected=resources.values().any(|value|match value {
        Value::View(view)=>(view.table_selection().is_some()||view.xy_selection().is_some()),
        Value::BogaeComposition(composition)=>composition.relations().iter().any(|r|r.view().table_selection().is_some()||r.view().xy_selection().is_some()),
        _=>false,
    });
    if !has_selected {return Ok(TypedTableProjection {projection,selections:Vec::new(),source:source_identity.into()});}
    let mut selections=Vec::new();
    for (ci,candidate) in projection.as_ref().and_then(|p|p["candidates"].as_array()).into_iter().flatten().enumerate() {
        let name=candidate["label"].as_str().ok_or("E_AUTHORED_BOGAE: 실제 보기 선언이 없습니다")?;
        let declaration=program.root_stmts.iter().find_map(|s|match s {
            Stmt::Definition{name:n,span,..} if n==name
                && candidate["artifact"]["identity"]["composition_declaration"]==json!([span.start,span.end])=>Some(span),_=>None,
        }).ok_or("E_AUTHORED_BOGAE: 실제 루트 보기 선언이 없습니다")?;
        if candidate["artifact"]["identity"]!=json!({"scope":"temporary_execution","module":source_identity,
            "composition_declaration":[declaration.start,declaration.end]}) {
            return Err("E_AUTHORED_BOGAE: 보기 생산과 원문 소유가 다릅니다".into());
        }
        let mut capture=|ri:Option<usize>,view:&ddonirang_lang::runtime::ViewValue|->Result<(),String>{
            if let Some(selection)=view.table_selection() {
                if !selection.same_source(source_identity) {return Err("E_AUTHORED_BOGAE: 열 선택의 원문 판본이 다릅니다".into());}
                selections.push((ci,ri,ObservationViewSelection::Table(selection.clone())));
            }
            if let Some(selection)=view.xy_selection() {
                if !selection.same_source(source_identity) {return Err("E_AUTHORED_BOGAE: XY 선택 원문 판본이 다릅니다".into());}
                selections.push((ci,ri,ObservationViewSelection::Xy(selection.clone())));
            }
            Ok(())
        };
        match resources.get(name) {
            Some(Value::View(view))=>capture(None,view)?,
            Some(Value::BogaeComposition(composition))=>{
                for (ri,relation) in composition.relations().iter().enumerate() {
                    capture(Some(ri),relation.view())?;
                    if matches!(relation,BogaeRelationValue::Primary{..}) {capture(None,relation.view())?;}
                }
            },
            _=>return Err("E_AUTHORED_BOGAE: 보기 생산 당시의 typed 값이 없습니다".into()),
        }
    }
    Ok(TypedTableProjection {projection,selections,source:source_identity.into()})
}

pub fn resolve_selected(program:&CanonProgram,source_identity:&str,
    resources:&HashMap<String,Value>,entry:Option<&str>)->Option<Json> {
    // Legacy callers must not silently erase a selected view into a full table.
    match resolve_selected_with_tables(program,source_identity,resources,entry) {
        Ok(packet) if packet.selections.is_empty()=>packet.projection,
        Ok(_)=>Some(json!({"schema":"ddn.bogae_projection.v1","status":"rejected",
            "diagnostic":{"code":"E_BOGAE_PROJECTION_INVALID","message":"선택 결과표에는 typed 운반 경로가 필요합니다"}})),
        Err(message)=>Some(json!({"schema":"ddn.bogae_projection.v1","status":"rejected",
            "diagnostic":{"code":"E_BOGAE_PROJECTION_INVALID","message":message}})),
    }
}

fn resolve_selected_raw(
    program: &CanonProgram,
    source_identity: &str,
    resources: &HashMap<String, Value>,
    entry: Option<&str>,
) -> Option<Json> {
    // F5-GA[PROJECTION_PRIMARY]: Composition relations are private and admitted with exactly one Primary; its clone preserves that carrier and relations() only borrows it.
    let mut candidates = Vec::new();
    // Only actual authored root bindings are eligible. Do not infer a view
    // from a project name, variable spelling, world data or legacy drawlist.
    for stmt in &program.root_stmts {
        let Stmt::Definition { name, span, .. } = stmt else {
            continue;
        };
        let Some(value) = resources.get(name) else {
            continue;
        };
        let identity = json!({"scope":"temporary_execution", "module":source_identity,
            "composition_declaration":[span.start,span.end]});
        let relations = match value {
            Value::View(_) => None, // RD-123: direct projection has no relation.
            Value::BogaeComposition(composition) => {
                let mut resolved = Vec::new();
                for (occurrence, relation) in composition.relations().iter().enumerate() {
                    let Some(location) = relation.source_location() else {
                        return Some(
                            json!({"schema":"ddn.bogae_projection.v1", "status":"rejected",
                            "diagnostic":{"code":"E_BOGAE_RELATION_UNRESOLVED", "message":"관계의 작성 위치를 해소할 수 없습니다"}}),
                        );
                    };
                    let (kind, order, basis, placement) = match relation {
                        BogaeRelationValue::Primary {..} => ("primary", Json::Null, Json::Null, Json::Null),
                        BogaeRelationValue::Overlay {order,basis_location,..} =>
                            ("overlay", json!(order), json!(basis_location), Json::Null),
                        BogaeRelationValue::Side {placement,..} =>
                            ("side", Json::Null, Json::Null, json!(placement.authored_name())),
                        BogaeRelationValue::Magnifier {..} =>
                            ("magnifier",Json::Null,Json::Null,Json::Null),
                    };
                    let identity = json!({"module":source_identity,
                        "composition_declaration":[span.start,span.end],
                        "authored_binding_location":location,"canonical_occurrence":occurrence});
                    let relation_id = crate::runtime_surface::sha256_hex(
                        format!("ddn.bogae_relation.v1\0{identity}").as_bytes());
                    // F5-GA[PROJECTION_OBJECT]: This local JSON object is never replaced or exposed mutably before the field insertions below.
                    let mut projected=json!({"identity":identity, "relation_id":relation_id,
                        "kind":kind, "order":order, "basis_location":basis, "placement":placement,
                        "view":crate::scene_value::project_view(relation.view()),
                        "time_policy":project_time(relation.time_policy())});
                    if let Some(policy)=relation.time_override() {
                        projected["time_override"]=project_time(policy);
                    }
                    // Presentation label only: exact authored root binding,
                    // never a source of identity or automatic view selection.
                    if let Some(label) = program.root_stmts.iter().find_map(|stmt| {
                        let Stmt::Definition {name,..}=stmt else {return None;};
                        match resources.get(name) {
                            Some(Value::BogaeRelation(bound)) if bound.source_location()==Some(location) => Some(name),
                            _ => None,
                        }
                    }) {
                        projected["authored_name"]=json!(label);
                    }
                    if let BogaeRelationValue::Magnifier {target,recipe,..}=relation {
                        let target=match crate::bogae_recipe::project_target(target) {
                            Ok(target)=>target,
                            Err(message)=>return Some(json!({"schema":"ddn.bogae_projection.v1","status":"rejected",
                                "diagnostic":{"code":"E_BOGAE_TARGET_UNSUPPORTED","message":message}})),
                        };
                        projected["target_digest"]=json!(crate::runtime_surface::sha256_hex(format!("ddn.bogae_target.v1\0{target}").as_bytes()));
                        // RD-135/520: only an explicitly authored typed scene
                        // element is pickable. Never guess coordinates from a
                        // list, variable name, recipe output or DOM callback.
                        // F5-GA[PROJECTION_READ]: Immutable JSON string-key reads return Null on missing/non-object data; these reads do not perform mutating Index insertion.
                        if target["kind"]=="scene_element" {
                            if let Some(object)=authored_object_identity(program,source_identity,resources,relation) {
                                projected["object_identity"]=object;
                            }
                            let primary=composition.relations().iter().find(|r|matches!(r,BogaeRelationValue::Primary {..})).expect("validated primary");
                            let view=crate::scene_value::project_view(primary.view());
                            projected["selection"]=primary_scene_element_selection(&view,&target)
                                .expect("checked scene-element target");
                        }
                        projected["target"]=target;
                        projected["recipe"]=json!({"kind":"sealed_named_callable","source":recipe.source_identity,
                            "definition":recipe.definition_name,"declaration":recipe.definition_span,
                            "parameter_type":recipe.parameter_type,"result_type":"보기값","seal":recipe.seal});
                    }
                    resolved.push(projected);
                }
                Some(resolved)
            }
            _ => continue,
        };
        let primary_view = match value {
            Value::View(view) => crate::scene_value::project_view(view),
            Value::BogaeComposition(composition) => crate::scene_value::project_view(composition.relations().iter()
                .find(|relation| matches!(relation,BogaeRelationValue::Primary {..})).expect("validated primary").view()),
            _ => unreachable!(),
        };
        let primary_time=match value {Value::View(view)=>view.time_policy(),
            Value::BogaeComposition(composition)=>composition.relations().iter()
                .find(|relation|matches!(relation,BogaeRelationValue::Primary {..})).expect("validated primary").time_policy(),
            _=>unreachable!()};
        let artifact = json!({"schema":"ddn.authored_bogae_projection.v1", "identity":identity,
            "view":primary_view, "relations":relations,
            "time_policy":project_time(primary_time),
            "lock":{"schema":"ddn.authored_bogae_projection.v1", "adapter":if primary_view["spatial_profile"]=="space3" {"scene_space3.r1"} else if primary_view["kind"] == "scene" {"scene_plane2.v1"} else {"result_table.v1"},
                "time":"common.v1", "diagnostic":"bogae.v1", "module_source":source_identity}});
        let digest = crate::runtime_surface::sha256_hex(
            format!("ddn.authored_bogae_projection.v1\0{artifact}").as_bytes(),
        );
        candidates.push(json!({"label":name,"artifact":artifact,"authored_digest":digest}));
    }
    if let Some(entry) = entry {
        candidates.retain(|candidate| candidate["label"].as_str() == Some(entry));
        if candidates.is_empty() {
            return Some(json!({"schema":"ddn.bogae_projection.v1","status":"rejected",
                "diagnostic":{"code":"E_BOGAE_ENTRY_UNRESOLVED","message":"명시 보개 시작이 보기값이나 보개짜임값으로 해소되지 않습니다"}}));
        }
    }
    if candidates.is_empty() {
        return None;
    }
    let selected = candidates.len() == 1;
    Some(
        json!({"schema":"ddn.bogae_projection.v1", "status":if selected {"ready"} else {"selection_required"},
        "candidates":candidates, "diagnostic":if selected {Json::Null} else {
            json!({"code":"E_BOGAE_SELECTION_REQUIRED","message":"보기나 보개짜임이 여러 개입니다. 명시 보개 시작을 지정하세요"})},
        "persistent_identity":false,"state_hash_participates":false}),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_exact_builtin_observations_are_deferred() {
        for name in ["표준보기.장면", "표준보기.결과표", "표준보개.으뜸", "표준보개.짜임", "표준보개.겹", "표준보개.곁", "표준보개.돋"] {
            assert!(super::is_observation_constructor(name));
        }
        for name in ["표준보기.사용자계산", "표준보개.짜임계산", "사용자.장면", "표준장면.원"] {
            assert!(!super::is_observation_constructor(name));
        }
    }
    #[test]
    fn named_entry_hook_views_observe_this_madi_without_world_writes() {
        let entry = "초기 := 2. 실행:움직씨 := { (시작)할때 { 위치 <- 초기. }. (매마디)마다 { 위치 <- 위치+1. }. }.";
        let view = "으뜸 := (보기=((장면=((요소들=차림[((중심=차림[위치,0],반지름=0.2) 표준장면.원)],가로범위=차림[0,10],세로범위=차림[-1,1]) 표준장면.평면)) 표준보기.장면)) 표준보개.으뜸. 짜임 := (관계들=차림[으뜸]) 표준보개.짜임.";
        let run = |source: &str| crate::runtime_surface::run_summary_from_supported_source_profile_with_update_name_diagnostic(
            source, "named-entry-view.ddn", Some(3), Some("v1-core-v25"), "실행", None).unwrap();
        let world_only = run(entry);
        let observed = run(&format!("{entry} {view}"));
        assert_eq!(observed["resources"]["fixed64"]["위치"], "5");
        assert_eq!(observed["state_hash"], world_only["state_hash"]);
        let candidates = observed["bogae_projection"]["candidates"].as_array().unwrap();
        let composition = candidates.iter().find(|row| row["label"] == "짜임").unwrap();
        assert_eq!(composition["artifact"]["view"]["elements"][0]["center"][0],
            serde_json::json!(Fixed64::from_i64(5).raw_i64().to_string()));
        let invalid = format!("{entry} 으뜸 := (보기=((장면=((요소들=차림[((중심=차림[미정,0],반지름=0.2) 표준장면.원)],가로범위=차림[0,10],세로범위=차림[-1,1]) 표준장면.평면)) 표준보기.장면)) 표준보개.으뜸.");
        assert!(crate::runtime_surface::run_summary_from_supported_source_profile_with_update_name_diagnostic(
            &invalid, "missing.ddn", Some(1), Some("v1-core-v25"), "실행", None).is_err());
    }
    use super::*;
    #[test]
    fn a3_rd136_time_policy_values_are_typed_and_not_world_resources() {
        // RD-136; A3-TIME-P1 optional constructor time pin is provisional.
        let source="기준 := (번호=0) 표준시간.고정마디. 차이 := (차이=-1) 표준시간.상대마디. 공통 := () 표준시간.공통.";
        let output=run(source,"time-values.ddn");
        assert!(output.patch.ops.is_empty());
    }
    #[test]
    fn a3_rd136_view_default_and_relation_override_have_distinct_owners() {
        let source="시간값 := (번호=0) 표준시간.고정마디. 보기 := (시간=시간값) 표준보기.결과표. 기본 := (보기=보기,시간=(() 표준시간.공통)) 표준보개.으뜸. 곁 := (보기=보기,자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,곁]) 표준보개.짜임.";
        let output=run(source,"time-owners.ddn");
        let projection=output.bogae_projection.unwrap();
        assert_eq!(projection["status"],"selection_required","value creation must not auto-activate one of two candidates");
        let candidate=projection["candidates"].as_array().unwrap().iter().find(|candidate|candidate["label"]=="짜임").unwrap();
        let rows=&candidate["artifact"]["relations"];
        assert_eq!(rows[0]["time_policy"],json!({"kind":"common"}));
        assert_eq!(rows[0]["time_override"],json!({"kind":"common"}));
        assert_eq!(rows[1]["time_policy"],json!({"kind":"fixed","madi":0}));
        assert!(rows[1].get("time_override").is_none());
    }
    #[test]
    fn a3_rd136_policy_rejects_fraction_negative_fixed_and_plain_pack() {
        use crate::ddn_runtime::{DdnProgram,DdnRunner};
        for source in ["시간 := (번호=-1) 표준시간.고정마디.",
            "시간 := (차이=0.5) 표준시간.상대마디.",
            "보기 := (시간=(종류:\"common\")) 표준보기.결과표."] {
            let program=DdnProgram::from_v25_source(source,"invalid-time.ddn").unwrap();
            let mut runner=DdnRunner::new(program,"매마디");
            let input=InputSnapshot {tick_id:0,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
                pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
            let error=runner.run_update(&NuriWorld::new(),&input,&HashMap::new()).err().expect("invalid time must fail");
            assert!(error.contains("E_BOGAE_TIME_POLICY_INVALID"),"{error}");
        }
    }
    use crate::ddn_runtime::{DdnProgram, DdnRunner};
    use ddonirang_core::platform::NuriWorld;
    use ddonirang_core::{Fixed64, InputSnapshot, InputSource};

    fn run(source: &str, path: &str) -> crate::ddn_runtime::DdnRunOutput {
        let program = DdnProgram::from_v25_source(source, path).unwrap();
        DdnRunner::new(program, "매마디")
            .run_update(
                &NuriWorld::new(),
                &InputSnapshot {
                    tick_id: 0,
                    dt: Fixed64::ONE,
                    keys_pressed: 0,
                    last_key_name: String::new(),
                    pointer_x_i32: 0,
                    pointer_y_i32: 0,
                    ai_injections: vec![],
                    net_events: vec![],
                    frame_source: InputSource::Person,
                    rng_seed: 0,
                },
                &HashMap::new(),
            )
            .unwrap()
    }

    #[test]
    fn scene_target_resolves_only_exact_authored_primary_element() {
        let source="(대상:장면요소값) 자세히:보기값:셈씨 := { 장면 := (요소들=차림[대상], 가로범위=차림[-2,2], 세로범위=차림[-2,2]) 표준장면.평면. 자세히 <- (장면=장면) 표준보기.장면. }. 점 := (중심=차림[0,0],반지름=0.1) 표준장면.원. 다른점 := (중심=차림[1,1],반지름=0.1) 표준장면.원. 장면 := (요소들=차림[점], 가로범위=차림[-2,2], 세로범위=차림[-2,2]) 표준장면.평면. 기본 := (보기=((장면=장면) 표준보기.장면)) 표준보개.으뜸. 첫돋 := (대상=점,보기법=자세히) 표준보개.돋. 둘째돋 := (대상=다른점,보기법=자세히) 표준보개.돋. 짜임 := (관계들=차림[기본,첫돋,둘째돋]) 표준보개.짜임.";
        let output=run(source,"picking.ddn");
        let relations=&output.bogae_projection.as_ref().unwrap()["candidates"][0]["artifact"]["relations"];
        assert_eq!(relations[1]["selection"]["element_indices"],json!([0]));
        assert_eq!(relations[2]["selection"]["element_indices"],json!([]));
        assert_eq!(relations[1]["target"]["element"],relations[0]["view"]["elements"][0]);
        assert_eq!(relations[1]["object_identity"]["label"],json!("점"));
        assert_ne!(relations[1]["object_identity"]["id"],relations[2]["object_identity"]["id"]);
        assert_eq!(output.bogae_projection,run(source,"renamed.ddn").bogae_projection,
            "file names are not object identity");
        assert!(output.patch.ops.is_empty());
    }

    #[test]
    fn sealed_named_magnifier_projects_target_and_recipe_without_world_patch() {
        let source="(대상:수) 확대:보기값:셈씨 := { 원 := (중심=차림[0,0], 반지름=대상) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 확대 <- (장면=장면) 표준보기.장면. }. 기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 자세히 := (대상=2, 보기법=확대) 표준보개.돋. 짜임 := (관계들=차림[기본,자세히]) 표준보개.짜임.";
        let output=run(source,"magnifier.ddn");
        let relation=&output.bogae_projection.as_ref().unwrap()["candidates"][0]["artifact"]["relations"][1];
        assert_eq!(relation["kind"],"magnifier");
        assert_eq!(relation["target"]["raw"],"8589934592");
        assert_eq!(relation["recipe"]["definition"],"확대");
        assert_eq!(relation["recipe"]["kind"],"sealed_named_callable");
        assert_eq!(relation["view"]["elements"][0]["radius"],"8589934592");
        assert!(relation.get("selection").is_none(),"a scalar must not invent a scene target");
        assert!(relation.get("object_identity").is_none(),"anonymous values must not invent an object identity");
        assert_eq!(output.bogae_projection,run(source,"renamed.ddn").bogae_projection);
        assert!(output.patch.ops.is_empty());
    }

    #[test]
    fn authored_circle_text_plane_projects_exact_geometry_without_world_patch() {
        let source="원 := (중심=차림[0,0], 반지름=2) 표준장면.원. 글 := (자리=차림[0,1], 내용=\"중심\") 표준장면.글. 장면 := (요소들=차림[원,글], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 보기 := (장면=장면) 표준보기.장면.";
        let output=run(source,"circle.ddn");
        let view=&output.bogae_projection.as_ref().unwrap()["candidates"][0]["artifact"]["view"];
        assert_eq!(view["kind"],"scene");
        assert_eq!(view["elements"][0]["radius"],"8589934592");
        assert_eq!(view["elements"][1]["content"],"중심");
        assert!(output.patch.ops.is_empty());
    }

    #[test]
    fn polygon_composition_copy_charges_all_four_vertices() {
        let prefix="도형 := (꼭짓점들=차림[차림[0,0],차림[2,0],차림[2,2],차림[0,2]]) 표준장면.다각형. 장면 := (요소들=차림[도형], 가로범위=차림[-1,3], 세로범위=차림[-1,3]) 표준장면.평면. ";
        let direct=run(&format!("{prefix}보기 := (장면=장면) 표준보기.장면."),"meter.ddn");
        let composed=run(&format!("{prefix}기본 := (보기=((장면=장면) 표준보기.장면)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임."),"meter.ddn");
        // Primary: element+four vertices=5; list:1; composition relation:1;
        // composition view clone: element+four vertices=5; common builtin
        // argument-list traversal:1. Total extra=13 (base dispatch separate).
        assert_eq!(composed.work_cost.counters.collection_tensor_element_visits
            -direct.work_cost.counters.collection_tensor_element_visits,13);
        assert_eq!(composed.work_cost.counters.bigint_rational_signed_magnitude_byte_visits,
            direct.work_cost.counters.bigint_rational_signed_magnitude_byte_visits);
    }

    #[test]
    fn temporary_primary_is_source_bound_not_path_bound_or_world_state() {
        let source = "짜임:보개짜임값 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임.";
        let a = run(source, "first.ddn");
        let b = run(source, "moved/renamed.ddn");
        assert_eq!(a.bogae_projection, b.bogae_projection);
        let projection = a.bogae_projection.unwrap();
        assert_eq!(projection["status"], "ready");
        assert_eq!(projection["persistent_identity"], false);
        assert_eq!(
            projection["candidates"][0]["artifact"]["relations"][0]["kind"],
            "primary"
        );
        assert!(a.patch.ops.is_empty());
        assert_ne!(
            Some(projection),
            run(&format!("{source}\n"), "first.ddn").bogae_projection
        );
    }

    #[test]
    fn no_name_or_order_heuristic_selects_multiple_authored_values() {
        let projection = run(
            "첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.",
            "x.ddn",
        )
        .bogae_projection
        .unwrap();
        assert_eq!(projection["status"], "selection_required");
        assert_eq!(
            projection["diagnostic"]["code"],
            "E_BOGAE_SELECTION_REQUIRED"
        );
        assert_eq!(projection["candidates"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn root_public_observation_keeps_original_source_binding() {
        let output = run("보기 := () 표준보기.결과표. (측정: 3) 내보임.", "root.ddn");
        assert_eq!(output.public_observations.len(), 1);
        assert_eq!(output.public_observations[0].value_text, "3");
        assert!(output.public_observations[0]
            .source_ref
            .starts_with("root.ddn#bytes="));
    }

    #[test]
    fn root_and_callable_observations_map_in_authored_order() {
        let output = run("(첫째: 1) 내보임. 기록:움직씨 := { (둘째: 2) 내보임. }. () 기록.", "mixed.ddn");
        assert_eq!(output.public_observations.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(), vec!["첫째", "둘째"]);
        assert!(output.public_observations.iter().all(|row| row.source_ref.starts_with("mixed.ddn#bytes=")));
    }

    #[test]
    fn overlay_and_side_preserve_authored_roles_and_validate_basis() {
        let prefix = "기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. ";
        let source = format!("{prefix}겹 := (보기=(() 표준보기.결과표), 기준=기본, 순서=1) 표준보개.겹. 곁 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,겹,곁]) 표준보개.짜임.");
        let projection = run(&source,"roles.ddn").bogae_projection.unwrap();
        assert_eq!(projection["status"],"ready");
        let relations = projection["candidates"][0]["artifact"]["relations"].as_array().unwrap();
        assert_eq!(relations.iter().map(|r| r["kind"].as_str().unwrap()).collect::<Vec<_>>(),vec!["primary","overlay","side"]);
        assert_eq!(relations[2]["placement"],"오른쪽");
        assert_eq!(relations[1]["authored_name"],"겹");
        assert_eq!(relations[2]["authored_name"],"곁");
        assert_ne!(relations[0]["relation_id"],relations[2]["relation_id"]);
    }

    #[test]
    fn rejects_foreign_primary_duplicate_overlay_order_and_relation() {
        let prefix = "기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 다른 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. ";
        for (suffix,code) in [
            ("겹 := (보기=(() 표준보기.결과표), 기준=다른, 순서=1) 표준보개.겹. 짜임 := (관계들=차림[기본,겹]) 표준보개.짜임.","E_BOGAE_OVERLAY_BASIS_INVALID"),
            ("겹 := (보기=(() 표준보기.결과표), 기준=기본, 순서=1) 표준보개.겹. 두겹 := (보기=(() 표준보기.결과표), 기준=기본, 순서=1) 표준보개.겹. 짜임 := (관계들=차림[기본,겹,두겹]) 표준보개.짜임.","E_BOGAE_OVERLAY_ORDER_DUPLICATE"),
            ("곁 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,곁,곁]) 표준보개.짜임.","E_BOGAE_RELATION_DUPLICATE"),
        ] {
            let program = DdnProgram::from_v25_source(&format!("{prefix}{suffix}"),"roles.ddn").unwrap();
            let error = DdnRunner::new(program,"매마디").run_update(&NuriWorld::new(),
                &InputSnapshot {tick_id:0,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0},
                &HashMap::new()).err().unwrap().to_string();
            assert!(error.contains(code),"{error}");
        }
    }

    #[test]
    fn activation_rejects_multiple_values_before_committed_madi() {
        let source = "값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.";
        let program = DdnProgram::from_v25_source(source, "atomic.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디").with_bogae_activation();
        let world = NuriWorld::new();
        let input = InputSnapshot {tick_id:0,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
            pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],
            frame_source:InputSource::Person,rng_seed:0};
        assert!(runner.run_update(&world,&input,&HashMap::new()).err().unwrap().to_string().contains("E_BOGAE_SELECTION_REQUIRED"));
        assert_eq!(serde_json::to_value(runner.work_cost_receipt()).unwrap()["counters"]["committed_madi"],0);
        // The same source remains valid in the pure constructor-evaluation tier.
        assert_eq!(run(source,"atomic.ddn").bogae_projection.unwrap()["candidates"].as_array().unwrap().len(),2);
    }

    #[test]
    fn explicit_entry_selects_exact_authored_value_without_world_change() {
        let source="첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표. 수값 := 2.";
        let world=NuriWorld::new();
        let input=InputSnapshot {tick_id:0,dt:Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
            pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
        let program=DdnProgram::from_v25_source(source,"explicit.ddn").unwrap();
        let mut runner=DdnRunner::new(program,"매마디").with_bogae_activation();
        assert!(runner.configure_bogae_entry("없는이름").is_err());
        runner.configure_bogae_entry("둘째").unwrap();
        let output=runner.run_update(&world,&input,&HashMap::new()).unwrap();
        let projection=output.bogae_projection.unwrap();
        assert_eq!(projection["status"],"ready");
        assert_eq!(projection["candidates"].as_array().unwrap().len(),1);
        assert_eq!(projection["candidates"][0]["label"],"둘째");
        assert!(output.patch.ops.is_empty());
        let program=DdnProgram::from_v25_source(source,"explicit.ddn").unwrap();
        let mut invalid=DdnRunner::new(program,"매마디").with_bogae_activation();
        invalid.configure_bogae_entry("수값").unwrap();
        assert!(invalid.run_update(&world,&input,&HashMap::new()).err().unwrap().to_string().contains("E_BOGAE_ENTRY_UNRESOLVED"));
        assert_eq!(serde_json::to_value(invalid.work_cost_receipt()).unwrap()["counters"]["committed_madi"],0);
    }
}
