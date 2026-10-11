//! Non-world projection of authored typed values. This is not a replay format
//! or a persistent identity: source + declaration owns temporary execution IDs.
use ddonirang_lang::runtime::{BogaeRelationValue, Value};
use ddonirang_lang::{CanonProgram, ExprKind, Stmt};
use serde_json::{json, Value as Json};
use std::collections::HashMap;

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

pub fn resolve_selected(
    program: &CanonProgram,
    source_identity: &str,
    resources: &HashMap<String, Value>,
    entry: Option<&str>,
) -> Option<Json> {
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
                    let mut projected=json!({"identity":identity, "relation_id":relation_id,
                        "kind":kind, "order":order, "basis_location":basis, "placement":placement,
                        "view":crate::scene_value::project_view(relation.view()),
                        "time_policy":{"kind":"common"}});
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
                        if target["kind"]=="scene_element" {
                            if let Some(object)=authored_object_identity(program,source_identity,resources,relation) {
                                projected["object_identity"]=object;
                            }
                            let primary=composition.relations().iter().find(|r|matches!(r,BogaeRelationValue::Primary {..})).expect("validated primary");
                            let view=crate::scene_value::project_view(primary.view());
                            let indices:Vec<_>=view["elements"].as_array().into_iter().flatten().enumerate()
                                .filter(|(_,element)|**element==target["element"])
                                .map(|(index,_)|index).collect();
                            projected["selection"]=json!({"kind":"primary_scene_element.v1","element_indices":indices});
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
        let artifact = json!({"schema":"ddn.authored_bogae_projection.v1", "identity":identity,
            "view":primary_view, "relations":relations,
            "time_policy":{"kind":"common"},
            "lock":{"schema":"ddn.authored_bogae_projection.v1", "adapter":if primary_view["kind"] == "scene" {"scene_plane2.v1"} else {"result_table.v1"},
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
    use super::*;
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
        let output = run("보기 := () 표준보기.결과표. (측정: 3) 보임.", "root.ddn");
        assert_eq!(output.public_observations.len(), 1);
        assert_eq!(output.public_observations[0].value_text, "3");
        assert!(output.public_observations[0]
            .source_ref
            .starts_with("root.ddn#bytes="));
    }

    #[test]
    fn root_and_callable_observations_map_in_authored_order() {
        let output = run("(첫째: 1) 보임. 기록:움직씨 := { (둘째: 2) 보임. }. () 기록.", "mixed.ddn");
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
