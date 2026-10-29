
    use super::*;
    #[test]
    fn equation_inputs_same_helper_checks_both_sides_and_constants() {
        let helper=include_str!("../../tests/fixtures/student_equation_helper.ddn");
        for (left,right,answer) in [("2*x","x+3","3"),("5*x","x+2","1/2"),
            ("y+3","7","4"),("7","x+3","4"),("x-x+2*x","4","2"),("3*x","1","1/3")] {
            let source=format!("{helper}\n((수식{{{left}}} =:= 수식{{{right}}}) 풀어검산하기) 보여주기.");
            let program=DdnProgram::from_v25_source(&source,"equation-inputs.ddn").unwrap();
            let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
            let text=&output.display_outputs.last().unwrap().text;
            assert!(text.contains("검산: 참")&&text.contains(&format!("값={answer}")),"{source}\n{text}");
        }
    }

    #[test]
    fn equation_inputs_preserve_exact_values_and_required_variable_sets() {
        let formula=|raw:&str|Formula{raw:raw.into(),dialect:FormulaDialect::Ascii,explicit_tag:true};
        for (left,right,name,num,den,left_has,right_has) in [
            ("2*x","x+3","x",3,1,true,true), ("y+3","7","y",4,1,true,false),
            ("7","x+3","x",4,1,false,true),("x-x+2*x","4","x",2,1,true,false),
            ("3*x","1","x",1,3,true,false)] {
            let Value::Pack(ref relation)=make_relation_pack(formula(left),formula(right)) else{unreachable!()};
            let Value::Pack(ref result)=eval_relation_solve_result(&[relation.clone()]).unwrap() else{unreachable!()};
            for (field,present) in [("왼쪽입력",left_has),("오른쪽입력",right_has)] {
                let Value::Pack(inputs)=&result[field] else{panic!("{field}")};
                assert_eq!(inputs.len(),usize::from(present));
                if present {
                    let exact=exact_numeric_from_value(&inputs[name]).unwrap().unwrap();
                    assert_eq!(exact.value,BigRational::new(BigInt::from(num),BigInt::from(den)));
                }
            }
        }
        for (left,right) in [("x+1","x+2"),("x+1","x+1"),("x^3","1")] {
            let Value::Pack(ref relation)=make_relation_pack(formula(left),formula(right)) else{unreachable!()};
            let Value::Pack(ref result)=eval_relation_solve_result(&[relation.clone()]).unwrap() else{unreachable!()};
            assert!(!result.contains_key("왼쪽입력")&&!result.contains_key("오른쪽입력"));
        }
    }
    #[test]
    fn equation_inputs_saved_project_restores_only_with_source_custody() {
        use crate::authored_project::{ProjectArtifact, SourceSnapshot, SourceFile};
        let root = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 관계 := 수식{3*x} =:= 수식{1}. 결과 <- (관계) 방정식풀기. 결과 보여주기. }.";
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),files:vec![
                SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"app/main.ddn".into(),text:body.into()}]
        },String::new()).unwrap();
        let saved=artifact.save().unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(saved.as_bytes()).unwrap();
        let make_runner=|| {
            let program=DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(),ParseMode::Strict,AgeTarget::Age3).unwrap();
            let mut runner=DdnRunner::new(program,"실행");runner.configure_project_entry("실행").unwrap();runner
        };
        let mut runner=make_runner();
        assert!(runner.tuple_type_plan.equation_result("결과"),"{:?}; {:?}",runner.tuple_type_plan,runner.program.numeric_constraint_plan);
        let output=runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        let mut world=DetNuri::new();
        world.apply_patch(&output.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        let snapshot=crate::runtime_surface::runtime_state_snapshot(&world,Vec::new(),1)["resources"].clone();
        let raw=crate::unit_wire::serialize_resource_snapshot_json(&snapshot);
        let parsed=crate::unit_wire::parse_snapshot_json(&raw).unwrap();
        let mut restored=NuriWorld::new();
        assert!(crate::unit_wire::restore_world_resources_snapshot(&mut restored,&parsed).is_err());
        let mut reopened=make_runner();
        let plan=reopened.saved_tuple_restore_plan().unwrap();
        crate::unit_wire::restore_world_resources_with_tuple_plan(&mut restored,&parsed,Some(&plan)).unwrap();
        assert_eq!(restored.state_hash(),world.world().state_hash());
        let output=reopened.run_update(&restored,&empty_input(),&HashMap::new()).unwrap();
        assert!(output.display_outputs.last().unwrap().text.contains("값=1/3"));
        let value=world.world().get_resource_value("결과").unwrap();
        let unrelated=SavedTupleRestorePlan{plan:crate::tuple_saved_type_plan::TupleSavedTypePlan::default()};
        assert!(unrelated.admit_resource("결과",&value).is_err());
        let mut altered=equation_saved_value::decode(&value).unwrap();
        let Value::Pack(fields)=&mut altered else{panic!("result")};
        let Some(Value::Pack(inputs))=fields.get_mut("왼쪽입력") else{panic!("inputs")};
        inputs.insert("x".into(),make_rational_pack_from_big_rational(&BigRational::from_integer(2.into())));
        assert!(equation_saved_value::encode(&altered).is_err());
    }

    #[test]
    fn learning_formula_errors_preserve_innermost_source_range() {
        for newline in ["\n","\r\n"] {
            for (body,marker,reason) in [
                ("(수식{x+1} 해서 () 풀기) 보여주기.","() 풀기","주입 키가 누락"),
                ("(수식{7} 해서 (x=4) 풀기) 보여주기.","(x=4) 풀기","주입 키가 여분"),
                ("((수식{sin(x)} =:= 수식{0}) 방정식풀기) 보여주기.","방정식풀기","E_SYMBOLIC_PARSE"),
                ("결과 := (수식{x} =:= 수식{1}) 방정식풀기.\n(수식{1/(x-x)} 해서 (주입=결과.왼쪽입력) 풀기) 보여주기.","(주입=결과.왼쪽입력) 풀기","0"),
            ] {
                let source=format!("// 한글 🍎{newline}{newline}{}",body.replace('\n',newline));
                let program=DdnProgram::from_v25_source(&source,"학습_진단.ddn").unwrap();
                let error=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                    .run_update_diagnostic(&NuriWorld::new(),&empty_input(),&HashMap::new()).err().expect("actual runtime rejection");
                let (_,message,span)=error.diagnostic_parts();
                assert!(message.contains(reason),"{message}");
                let (start,end)=span.expect("source range retained");
                let slice=source.get(start..end).expect("UTF-8 boundary");
                assert!(slice.contains(marker),"{slice:?}: {message}");
                assert!(!slice.contains("보여주기")&&!slice.contains("🍎"),"outer call must not replace inner source: {slice}");
            }
        }
    }

    // The authored helper is ordinary DDN; no test-side lowering.
    const EXACT_FORMULA_STUDENT_HELPER:&str=r#"
(관계:묶음~을) 풀어검산하다{풀어검산하기}:묶음:셈씨 := {
    결과 := (관계) 방정식풀기.
    { 결과.__풀이결과종류 == "성공" }이면 {
        확인 := (관계.왼쪽 해서 (x=결과.값) 풀기) == (관계.오른쪽 해서 () 풀기).
        풀어검산하다 <- (풀이: 결과, 검산: 확인).
    } 아니면 {
        풀어검산하다 <- (풀이: 결과).
    }.
}.
"#;
    #[test]
    fn exact_formula_student_helper_preserves_fraction_answers() {
        let helper=EXACT_FORMULA_STUDENT_HELPER;
        for (left,right,answer) in [("2*x","1","1/2"),("3*x","1","1/3"),("x+3","7","4")] {
            let source=format!("{helper}\n((수식{{{left}}} =:= 수식{{{right}}}) 풀어검산하기) 보여주기.");
            let program=DdnProgram::from_v25_source(&source,"exact-formula-student.ddn").unwrap();
            assert_eq!(program.numeric_constraint_plan.as_ref().unwrap().exact_formula_calls.len(),2,"{source}\n{:#?}",program.program);
            let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
            let text=&output.display_outputs.last().unwrap().text;
            assert!(text.contains("검산: 참"),"{text}");
            assert!(text.contains(&format!("값={answer}")),"{text}");
        }
    }
    #[test]
    fn exact_formula_student_helper_preserves_solver_failure_reasons() {
        let helper=EXACT_FORMULA_STUDENT_HELPER;
        for (left,right,reason) in [("x+1","x+2","no_solution"),("x+1","x+1","non_unique"),("x^3","1","unsupported")] {
            let source=format!("{helper}\n((수식{{{left}}} =:= 수식{{{right}}}) 풀어검산하기) 보여주기.");
            let program=DdnProgram::from_v25_source(&source,"exact-formula-failure.ddn").unwrap();
            let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
            let text=&output.display_outputs.last().unwrap().text;
            assert!(text.contains(reason),"{text}");
            assert!(!text.contains("검산:"),"{text}");
        }
        // DDN admission succeeds; the solver's symbolic parser rejects this
        // expression during execution. Preserve that actual error boundary.
        let source=format!("{helper}\n((수식{{sin(x)}} =:= 수식{{0}}) 풀어검산하기) 보여주기.");
        let program=DdnProgram::from_v25_source(&source,"exact-formula-syntax.ddn").unwrap();
        let error=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).err().expect("symbolic parse rejection during execution");
        assert!(error.contains("E_SYMBOLIC_PARSE"),"{error}");
    }
    #[test]
    fn exact_formula_pack_fields_do_not_forge_source_relation_ownership() {
        let helper=EXACT_FORMULA_STUDENT_HELPER;
        let fake="((왼쪽: 수식{2*x}, 오른쪽: 수식{1}) 풀어검산하기) 보여주기.";
        let source=format!("{helper}\n{fake}");
        let program=DdnProgram::from_v25_source(&source,"exact-formula-fake.ddn").unwrap();
        assert!(program.numeric_constraint_plan.as_ref().unwrap().exact_formula_calls.is_empty());
        let source=format!("{helper}\n((수식{{2*x}} =:= 수식{{1}}) 풀어검산하기) 보여주기.\n{fake}");
        let error=DdnProgram::from_v25_source(&source,"exact-formula-mixed.ddn").err().expect("mixed provenance rejected");
        assert!(error.contains("방정식 생성자"),"{error}");
    }
    #[test]
    fn vc04_read_snapshot_runtime_backstop_walks_deep_values_and_captures(){
        let table=Arc::new(ddonirang_core::nominal::NominalSlotTableV1::from_slots([1;32],&[],2_000_000,1_000_000).unwrap());
        let view=Arc::new(ddonirang_lang::runtime::FrozenSubjectViewV1::from_resolved_slots(table,BTreeMap::new()));
        let program=DdnProgram::from_v25_source("","guard-only.ddn").unwrap();
        let world=NuriWorld::new();let defaults=HashMap::new();
        let mut ctx=EvalContext::new(&program,&world,&defaults,None,InputState::new(0,0),0,0,None);
        let mut nested=Value::FrozenSubjectViewV1(view.clone());
        for _ in 0..20_000{nested=Value::List(vec![nested]);}
        let captured=Value::Lambda(LambdaValue{id:0,param:"값".into(),body:Expr::new(0,ddonirang_lang::Span{start:0,end:0},ExprKind::Literal(Literal::Bool(true))),
            captured:HashMap::from([("보관".into(),Value::FrozenSubjectViewV1(view))])});
        for value in [nested,captured] {
            assert!(value.contains_transient_subject_view());
            let error=ctx.set_source_resource("보관",value,0).unwrap_err();
            assert!(error.message().contains("E_VC04_READ_SNAPSHOT_ESCAPE"));
            assert!(ctx.patch_ops.is_empty());assert!(ctx.get_resource("보관").is_none());
        }
    }
    use crate::runtime_surface::DEFAULT_UPDATE_NAME;
    use ddonirang_core::platform::DetNuri;
    use ddonirang_core::{Nuri, KEY_A, KEY_D};
    use ddonirang_lang::runtime::Value as RuntimeValue;
    use std::sync::{Mutex, MutexGuard};

    const INPUT_KEY_Z: u64 = 1 << 7;

    #[test]
    fn vc13_tuple_raw_source_preserves_alias_and_static_positions() {
        let source=r#"검사:참거짓:셈씨 := {
            값 <- (1, "글", 참). 복사 <- 값.
            값[0] <- 2.
            범위값 <- 값[0..2]. 빈값 <- 값[1..1].
            검사 <- 복사[0] == 1 그리고 값[0] == 2 그리고 범위값[1] == "글" 그리고 빈값 == ().
        }.
        결과 <- () 검사."#;
        let program=DdnProgram::from_v25_source(source,"vc13-tuple.ddn").unwrap();
        let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["결과"],Value::Bool(true));
    }

    #[test]
    fn b01_inferred_absence_adopts_source_numeric_owner() {
        for (kind,initial,expected) in [("바른수","1","2"),("셈수","1.5","2.5"),("나눔수","1/2","3/2")] {
            let source=format!("계산:{kind}:셈씨 := {{ 값 <- 없음. 값 <- {initial}. 계산 <- 값 + 1. }}. (() 계산) 보여주기.");
            let program=DdnProgram::from_v25_source(&source,"first-none.ddn").unwrap();
            let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
            assert_eq!(output.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>(),vec![expected],"{source}");
        }
        let source="계산:바른수:셈씨 := { 값 <- 없음. 값 <- 1. 값 <- 1.5. 계산 <- 값. }. (() 계산) 보여주기.";
        assert!(DdnProgram::from_v25_source(source,"first-none-conflict.ddn").is_err(),"later canonical kinds must not be retagged");
    }

    #[test]
    fn b01_r1_2_host_scene_and_collection_source_bridge() {
        let source="(매마디)마다 { 프레임수 <- 프레임수 + 1. 프레임수 보여주기. }.";
        assert!(DdnProgram::from_v25_source(source,"unhosted.ddn").is_err(),"plain source factory must not invent a host");
        let hosted=|source:&str|DdnProgram::from_supported_source_profile_with_mode(source,"hosted.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25);
        let mut defaults=HashMap::new();crate::runtime_surface::seed_bogae_defaults(&mut defaults);
        let mut runner=DdnRunner::new(hosted(source).unwrap(),DEFAULT_UPDATE_NAME);
        let mut world=DetNuri::new();
        for tick in 0..2 {
            let output=runner.run_update(world.world(),&empty_input(),&defaults).unwrap();
            assert_eq!(output.resources["프레임수"],Value::Fixed64(Fixed64::from_i64(tick+1)));
            world.apply_patch(&output.patch,tick as u64,&mut ddonirang_core::signals::VecSignalSink::default());
        }
        for source in [
            "채비 { 프레임수:수 := 0. }. 프레임수 <- 프레임수 + 1.5. 프레임수 보여주기.",
            "(프레임수:수) 더함:수:셈씨 := { 더함 <- 프레임수 + 1.5. }. ((프레임수=0) 더함) 보여주기.",
        ] {let output=DdnRunner::new(hosted(source).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();assert_eq!(output.display_outputs.last().unwrap().text,"1.5");}
        // The existing runner prepares root 채비 before root statements.
        // Preserve that source initialization, rather than invent host-first.
        let prepared="프레임수 보여주기. 채비 { 프레임수:수 := 0. }.";
        let output=DdnRunner::new(hosted(prepared).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();
        assert_eq!(output.display_outputs[0].text,"0");
        for bad in ["{ 참 }이면 { 프레임수 <- 0.5. }. 프레임수 보여주기.",
            "{ 참 }이면 { 채비 { 프레임수:수 := 0.5. }. }. 프레임수 보여주기."] {
            assert!(hosted(bad).is_err(),"incompatible possible world source must not borrow a host kind: {bad}");
        }
        assert!(hosted("값:바른수 := 2147483648. 값 보여주기.").is_err(),"constant overflow must fail before a saturating carrier constructor");
        let mut maximal=DetNuri::new();
        let mut patch=Patch::default();patch.ops.push(PatchOp::SetResourceFixed64{tag:"프레임수".into(),value:Fixed64::from_raw_i64(2147483647i64<<32)});
        maximal.apply_patch(&patch,0,&mut ddonirang_core::signals::VecSignalSink::default());let before=maximal.world().state_hash();
        assert!(DdnRunner::new(hosted(source).unwrap(),DEFAULT_UPDATE_NAME).run_update(maximal.world(),&empty_input(),&defaults).is_err());assert_eq!(maximal.world().state_hash(),before);
        let good="그림:장면요소값:셈씨 := { n <- 0. t <- n/10. 그림 <- (중심=차림[t,0],반지름=1) 표준장면.원. }. (() 그림) 보여주기.";
        DdnRunner::new(hosted(good).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();
        for bad in [
            "값:나눔수 := 1. ((중심=차림[값,0],반지름=1) 표준장면.원) 보여주기.",
            "n:바른수 := 1. t <- n/10. ((중심=차림[t,0],반지름=1) 표준장면.원) 보여주기.",
        ] {assert!(hosted(bad).is_err(),"{bad}");}
        let good="계산:바른수:셈씨 := { 결과 <- (차림=차림[0],시작=0,끝=0) 토막내기. 결과 <- (결과,1) 추가. 계산 <- 결과[0]+1. }. (() 계산) 보여주기.";
        let output=DdnRunner::new(hosted(&good).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();assert_eq!(output.display_outputs.last().unwrap().text,"2");
        for bad in [
            "계산:바른수:셈씨 := { 결과 <- () 차림. 결과 <- (결과,1) 추가. 계산 <- 결과[0]+1. }. (() 계산) 보여주기.",
            "계산:바른수:셈씨 := { 근사:셈수 := 1. 결과 <- (차림=차림[0],시작=0,끝=0) 토막내기. 결과 <- (결과,근사) 추가. 계산 <- 결과[0]+1. }. (() 계산) 보여주기.",
        ] {assert!(hosted(bad).is_err(),"independent source T required: {bad}");}
    }

    #[test]
    fn b01_r1_2_scene_preserves_issued_kinds_and_all_uses() {
        let hosted=|source:&str|DdnProgram::from_supported_source_profile_with_mode(source,"scene-profile.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25);
        let mut defaults=HashMap::new();crate::runtime_surface::seed_bogae_defaults(&mut defaults);
        let good="값:바른수 := 2147483647. ((반지름=1,중심=차림[값,0]) 표준장면.원) 보여주기.";
        let program=hosted(good).unwrap();
        assert!(program.numeric_constraint_plan.as_ref().unwrap().bindings.values().any(|kind|kind==&TypeRef::Named("바른수".into())));
        DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();
        for bad in [
            "값:큰바른수 := 1. ((중심=차림[값,0],반지름=1) 표준장면.원) 보여주기.",
            "값 <- 1@m. ((중심=차림[값,0],반지름=1) 표준장면.원) 보여주기.",
            "값:나눔수 := 0.5. 점들 <- 차림[차림[값,0]]. 점들 <- (점들,차림[0,값]) 추가. 번호:바른수 := 1. ((중심=점들[번호],반지름=1) 표준장면.원) 보여주기.",
            "값 <- 1@m. 점들 <- 차림[차림[값,0]]. 점들 <- (점들,차림[0,값]) 추가. 번호:바른수 := 1. ((중심=점들[번호],반지름=1) 표준장면.원) 보여주기.",
            "그림:장면요소값:셈씨 := { 번호 <- 0. 자료 <- 차림[1]. 자료[번호] 보여주기. x <- 번호+0. 그림 <- (중심=차림[x,0],반지름=1) 표준장면.원. }. (() 그림) 보여주기.",
        ] {
            let error=hosted(bad).err().expect("unsupported typed kinds or conflicting source uses must reject");
            assert!(error.contains("E_TYPE_MISMATCH"),"{bad}: {error}");
        }
        let mixed_pair="정수:바른수 := 1. 근사:셈수 := 1.5. 점들 <- 차림[차림[정수,근사]]. 점들 <- (점들,차림[근사,정수]) 추가. 번호:바른수 := 1. ((중심=점들[번호],반지름=1) 표준장면.원) 보여주기.";
        DdnRunner::new(hosted(mixed_pair).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&defaults).unwrap();
    }

    #[test]
    fn b01_r1_2_collection_seed_loop_role_order_and_bounds() {
        let hosted=|source:&str|DdnProgram::from_supported_source_profile_with_mode(source,"collection-profile.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25);
        let good="계산:바른수:셈씨 := { 값 <- (끝=0,차림=차림[0],시작=0) 토막내기. 번호 <- 0. { 번호 < 3 }인 동안 { 값 <- (요소=번호,차림=값) 추가. 번호 <- 번호+1. }. 계산 <- 값[2]. }. (() 계산) 보여주기.";
        let output=DdnRunner::new(hosted(good).unwrap(),DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.display_outputs.last().unwrap().text,"2");
        for range in ["시작=0,끝=3","시작=2,끝=1"] {
            let source=format!("9 보여주기. ((차림=차림[1,2],{range}) 토막내기) 보여주기.");
            let world=NuriWorld::new();let before=world.state_hash();
            let error=DdnRunner::new(hosted(&source).unwrap(),DEFAULT_UPDATE_NAME).run_update(&world,&empty_input(),&HashMap::new()).err().expect("slice must not clip or publish");
            assert!(error.contains("E_SELECTION_BOUNDS"),"{error}");assert_eq!(world.state_hash(),before);
        }
        let bad="계산:바른수:셈씨 := { 값 <- (차림=차림[0],시작=0,끝=0) 토막내기. 근사:셈수 := 1. { 참 }이면 { 값 <- (값,근사) 추가. }. 계산 <- 값[0]. }. (() 계산) 보여주기.";
        assert!(hosted(bad).is_err(),"all possible incoming element owners must match, not just the initial seed");
    }

    #[test]
    fn b01_r1_2_append_all_incoming_kinds_not_only_initial_list() {
        let source="계산:참거짓:셈씨 := { 근사:셈수 := 1. 값 <- 차림[1]. 값 <- (값,근사) 추가. 위치:바른수 := 1. 계산 <- 값[위치] == 1. }. (() 계산) 보여주기.";
        let world=NuriWorld::new();let before=world.state_hash();
        let result=DdnProgram::from_supported_source_profile_with_mode(source,"append-owner.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25)
            .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&world,&empty_input(),&HashMap::new()));
        assert!(result.is_err(),"a typed 셈수 append must not inherit the initial 바른수 list's owner at a dynamic selection");
        assert_eq!(world.state_hash(),before);
    }

    #[test]
    fn b01_r1_2_collection_flow_survives_alias_branch_and_cached_call() {
        for source in [
            "계산:참거짓:셈씨 := { 근사:셈수 := 1. 값 <- 차림[1]. 별칭 <- 값. { 참 }이면 { 값 <- (값,근사) 추가. }. 위치:바른수 := 0. 계산 <- 별칭[위치] == 1. }. (() 계산) 보여주기.",
            "(자료:차림, 위치:바른수) 검사:참거짓:셈씨 := { 검사 <- 자료[위치] == 1. }. 근사:셈수 := 1. 처음 <- 차림[1]. (처음,0) 검사 보여주기. 다음 <- (처음,근사) 추가. (다음,1) 검사 보여주기.",
            "계산:참거짓:셈씨 := { 근사:셈수 := 1. 값 <- 차림[1]. 위치:바른수 := 0. 앞 <- 값[위치]. 값 <- (값,근사) 추가. 계산 <- 앞 == 1. }. (() 계산) 보여주기.",
        ] {
            let world=NuriWorld::new();let before=world.state_hash();
            let result=DdnProgram::from_v25_source(source,"all-incoming.ddn")
                .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&world,&empty_input(),&HashMap::new()));
            assert!(result.is_err(),"all independent incoming kinds must survive cached reads: {source}");
            assert_eq!(world.state_hash(),before);
        }
        // A collection need not issue homogeneous T merely to be displayed.
        let source="계산:차림:셈씨 := { 값 <- 차림[1]. 근사:셈수 := 1.5. 값 <- (값,근사) 추가. 계산 <- 값. }. (() 계산) 보여주기.";
        let program=DdnProgram::from_v25_source(source,"heterogeneous-display.ddn").unwrap();
        DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        let source="계산:참거짓:셈씨 := { 값 <- 차림[1]. 근사:셈수 := 1. (값,근사) 추가. 위치:바른수 := 0. 계산 <- 값[위치] == 1. }. (() 계산) 보여주기.";
        let program=DdnProgram::from_v25_source(source,"discarded-append.ddn").unwrap();
        let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.display_outputs.last().unwrap().text,"참","a discarded pure append is not a binding write");
        let source="계산:글:셈씨 := { 값 <- 차림[\"가\"]. 값 <- (값,\"나\") 추가. 위치:바른수 := 1. 계산 <- 값[위치]+\"!\". }. (() 계산) 보여주기.";
        let program=DdnProgram::from_v25_source(source,"text-collection.ddn").unwrap();
        let output=DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.display_outputs.last().unwrap().text,"나!");
    }

    #[test]
    fn b01_required_labs_admit_unmodified_authored_modules() {
        // Exact repository raw modules also compared byte-for-byte with R44
        // by the release CLI3/WASM gate. No test-only source transformation.
        for (entry,project,raw,root) in [
            ("alpha3_double_pendulum_lab.ddn","alpha3_double_pendulum_lab.project.ddn",
             include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha3_double_pendulum_lab.ddn"),
             include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha3_double_pendulum_lab.project.ddn")),
            ("alpha4_lrl_lab.ddn","alpha4_lrl_lab.project.ddn",
             include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha4_lrl_lab.ddn"),
             include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha4_lrl_lab.project.ddn")),
        ] {
            let request=serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
                "source_snapshot":{"root_path":project,"entry_path":entry,"files":[
                    {"path":project,"text":root},{"path":entry,"text":raw},
                    {"path":"alpha4_drawing_common.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha4_drawing_common.ddn")}]}});
            let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
            let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
            let program=DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(),ParseMode::Strict,default_age_target())
                .unwrap_or_else(|error|panic!("{entry}: {error}"));
            let mut runner=DdnRunner::new(program,owned.sources().entry_declaration().name());
            runner.configure_project_entry(owned.sources().entry_declaration().name()).unwrap();
            let mut defaults=HashMap::new();crate::runtime_surface::seed_bogae_defaults(&mut defaults);
            let mut world=DetNuri::new();
            for tick in 0..40 {
                let output=runner.run_update(world.world(),&empty_input(),&defaults).unwrap_or_else(|error|panic!("{entry} tick{tick}: {error}"));
                world.apply_patch(&output.patch,tick,&mut ddonirang_core::signals::VecSignalSink::default());
            }
        }
    }

    #[test]
    fn b01_tetris_integer_exception_keeps_wrong_kind_rejection() {
        // Owner's narrow source exception does not widen the numeric language.
        let raw=include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn");
        for wrong in ["가로값:셈수 := 1.","가로값:나눔수 := 0.5.","가로값 := 0.5."] {
            let source=format!("{raw}\n{wrong} ((칸들=차림[차림[0,0]],가로=가로값,세로=0) 칸옮김) 보여주기.");
            let error=DdnProgram::from_supported_source_profile_with_mode(&source,"tetris-wrong-kind.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25)
                .err().expect("real Tetris integer pin must reject wrong-kind/fraction");
            assert!(error.contains("E_TYPE_MISMATCH"),"{error}");
        }
    }

    #[test]
    fn b01_literal_tuple_source_defaults_and_canon_roundtrip() {
        use ddonirang_core::platform::RecordFieldType as R;
        let source = "정수묶음 <- (1,\"글\",참). 소수묶음 <- (1.0,\"글\",참). 합묶음 <- (1+2,0.1+0.2). 복사 <- 정수묶음[0..2]. 정수묶음[0] <- 2. 정수묶음[0] 보여주기.";
        for source in [source.to_owned(), ddonirang_lang::parse_v25_and_normalize(
            source, "b01.ddn", ddonirang_lang::NormalizationLevel::N1).unwrap()] {
            let program = DdnProgram::from_v25_source(&source, "b01.ddn").unwrap();
            let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
            let mut world = DetNuri::new();
            world.apply_patch(&output.patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
            for (name, kinds) in [("정수묶음", vec!["바른수","글","참거짓"]),
                ("소수묶음",vec!["셈수","글","참거짓"]), ("합묶음",vec!["바른수","셈수"]), ("복사",vec!["바른수","글"])] {
                let Some(ResourceValue::Tuple { ref element_types, .. }) = world.world().get_resource_value(name) else { panic!("{name}"); };
                assert_eq!(*element_types, kinds.into_iter().map(|name| R::Named { name:name.into() }).collect::<Vec<_>>());
            }
        }
    }

    #[test]
    fn n1_rational_integral_arithmetic_keeps_issued_owner() {
        let source="채비 { 값:나눔수 := 0. 합:나눔수 := 값+0. 곱:나눔수 := 값*5. 차:나눔수 := 값-값. }. (h:나눔수) 통과:나눔수:셈씨 := { 통과 <- h. }. 결과 <- (h=10*값-5*값*값) 통과.";
        let output=DdnRunner::new(DdnProgram::from_v25_source(source,"rational-integral-owner.ddn").unwrap(),DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        for name in ["값","합","곱","차","결과"] {
            assert!(matches!(&output.resources[name],Value::Rational(_)),"{name}: {:?}",output.resources[name]);
            let exact=exact_numeric_from_value(&output.resources[name]).unwrap().unwrap();
            assert_eq!(exact.value,BigRational::from_integer(BigInt::from(0)));
        }
    }

    #[test]
    fn b01_exact_expected_decimal_uses_original_digits_and_retains_rational_owner() {
        let source = "채비 { 합:나눔수 := 0.1+0.2. 자리:나눔수 := 0.10000000001. 정수몫:나눔수 := 4/2. }. 합 보여주기. 자리 보여주기. 정수몫 보여주기.";
        let output = DdnRunner::new(DdnProgram::from_v25_source(source,"b01-exact.ddn").unwrap(), DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        for (name, numerator, denominator) in [("합","3","10"),("자리","10000000001","100000000000"),("정수몫","2","1")] {
            let value=&output.resources[name];
            assert_eq!(numeric_pack_kind(value),Some(NUMERIC_KIND_RATIONAL));
            let exact=exact_numeric_from_value(value).unwrap().unwrap();
            assert_eq!(exact.value.numer().to_string(),numerator);
            assert_eq!(exact.value.denom().to_string(),denominator);
        }
        let bad="근사:셈수 := 0.1. 정확:나눔수 := 근사.";
        let world=NuriWorld::new(); let before=world.state_hash();
        let error=DdnProgram::from_v25_source(bad,"b01-no-retag.ddn")
            .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                .run_update(&world,&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
            .err().expect("a canonical approximate value must not become rational");
        assert!(error.contains("E_TYPE_MISMATCH"),"{error}");
        assert_eq!(world.state_hash(),before);
    }

    #[test]
    fn b01_typed_numeric_comparisons_reject_cross_kind_without_publication() {
        for op in ["==", "!=", "<", "<=", ">", ">="] {
            for (left, right) in [("정확", "근사"), ("근사", "정확"), ("정수", "근사")] {
                let source = format!("채비 {{ 정확:나눔수 := 1.0. 근사:셈수 := 1.0. 정수:바른수 := 1. }}. 9 보여주기. ({left} {op} {right}) 보여주기.");
                let world = NuriWorld::new(); let before = world.state_hash();
                // N1 can prove this contradiction at source admission. Keep
                // the same explicit cross-kind rejection and no-publication
                // invariant, without requiring an invalid VM to be created.
                let diagnostic = DdnProgram::from_v25_source(&source, "typed-compare.ddn")
                    .and_then(|program|DdnRunner::new(program, DEFAULT_UPDATE_NAME)
                        .run_update(&world,&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                    .err().expect(&source);
                assert!(diagnostic.contains("E_NUMERIC_COMPARISON_TYPE_MISMATCH"), "{source}: {diagnostic}");
                assert!(diagnostic.contains("자동 변환"), "{diagnostic}");
                assert_eq!(world.state_hash(), before);
            }
        }
    }

    #[test]
    fn b01_typed_numeric_comparisons_keep_parameter_result_and_tuple_owners() {
        for source in [
            "(왼값:나눔수,오른값:셈수) 비교:참거짓:셈씨 := { 비교 <- 왼값 == 오른값. }. 정확:나눔수 := 0.5. 근사:셈수 := 0.5. ((왼값=정확,오른값=근사) 비교) 보여주기.",
            "(원값:나눔수) 정확값:나눔수:셈씨 := { 정확값 <- 원값. }. 정확:나눔수 := 0.5. 근사:셈수 := 0.5. (((원값=정확) 정확값) == 근사) 보여주기.",
            "정수:바른수 := 1. 근사:셈수 := 1.0. 묶음 <- (정수,근사). (묶음[0] == 묶음[1]) 보여주기.",
            "근사:셈수 := 0.5. (((1,2) 나눔수) == 근사) 보여주기.",
            "(((2,1) 나눔수) == ((\"2\") 큰바른수)) 보여주기.",
        ] {
            let diagnostic=DdnProgram::from_v25_source(source,"compare-owner.ddn")
                .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                    .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                .err().expect(source);
            assert!(diagnostic.contains("E_NUMERIC_COMPARISON_TYPE_MISMATCH"), "{source}: {diagnostic}");
        }
    }

    #[test]
    fn b01_typed_numeric_negation_preserves_comparison_owner_and_canon() {
        for op in ["==", "!=", "<", "<=", ">", ">="] {
            for (left,right) in [("(-정확)","(-근사)"),("정확","(-근사)"),("(-정확)","근사"),("(-(-정확))","근사")] {
                let source=format!("정확:나눔수 := 0.5. 근사:셈수 := 0.5. 9 보여주기. ({left} {op} {right}) 보여주기.");
                let canon=ddonirang_lang::parse_v25_and_normalize(&source,"negative-owner.ddn",ddonirang_lang::NormalizationLevel::N1).unwrap();
                assert_eq!(ddonirang_lang::parse_v25_and_normalize(&canon,"negative-owner.ddn",ddonirang_lang::NormalizationLevel::N1).unwrap(),canon);
                for text in [&source,&canon] {
                    let error=DdnProgram::from_v25_source(text,"negative-owner.ddn")
                        .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                            .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                        .err().expect(text);
                    assert!(error.contains("E_NUMERIC_COMPARISON_TYPE_MISMATCH"),"{text}: {error}");
                }
            }
        }
        for source in [
            "정확:나눔수 := 0.5. 근사:셈수 := 0.5. 복사 := 근사. (정확 == (-복사)) 보여주기.",
            "정확:나눔수 := 0.5. 근사:셈수 := 0.5. 복사 <- 근사. (정확 == (-복사)) 보여주기.",
        ] {
            let error=DdnProgram::from_v25_source(source,"alias-owner.ddn")
                .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                    .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                .err().expect(source);
            assert!(error.contains("E_NUMERIC_COMPARISON_TYPE_MISMATCH"),"{source}: {error}");
        }
        for source in [
            "왼값:셈수 := 0.5. 오른값:셈수 := 0.5. ((-왼값) == (-오른값)) 보여주기.",
            "왼값:나눔수 := 0.5. 오른값:나눔수 := 0.5. ((-왼값) == (-오른값)) 보여주기.",
            "원본:나눔수 := 1. 음수:나눔수 := -원본. 복사 := -원본. (음수 == 복사) 보여주기.",
            "((-1) == (-1.0)) 보여주기. ((-(1/2)) == (-0.5)) 보여주기.",
        ] {
            DdnRunner::new(DdnProgram::from_v25_source(source,"negative-controls.ddn").unwrap(),DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        }
    }

    #[test]
    fn b01_static_plan_bag_transports_independent_source_kind_and_seeded_loops() {
        for (source,expected) in [
            ("가방 <- (0,차림[2]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값+1) 보여주기.",serde_json::json!(["3"])),
            ("가방 <- (0,차림[2]) 무작위가방.만들기. 미리 <- (가방,1) 무작위가방.미리보기. (미리[0]+1) 보여주기.",serde_json::json!(["3"])),
            ("가방 <- (0,차림[0]) 무작위가방.만들기. 별칭 <- 가방. 자료 <- 차림[7]. 반복수 <- 0. { 반복수 < 3 }인 동안 { 뽑음 <- (별칭) 무작위가방.꺼내기. 별칭 <- 뽑음.가방. 자료[뽑음.값] 보여주기. 반복수 <- 반복수+1. }.",serde_json::json!(["7","7","7"])),
            ("가방 <- (0,차림[\"가\"]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값+\"나\") 보여주기. (뽑음.값==\"가\") 보여주기. (뽑음.값==1) 보여주기.",serde_json::json!(["가나","참","거짓"])),
            ("가방 <- (0,차림[참]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값==참) 보여주기.",serde_json::json!(["참"])),
        ] {
            let result=crate::runtime_surface::run_summary_from_supported_source_profile(source,"bag-source-kind.ddn",Some(1),Some("v1-core-v25"))
                .unwrap_or_else(|error|panic!("{source}: {error}"));
            assert_eq!(result["output_display"]["texts"],expected,"{source}");
        }
    }

    #[test]
    fn b01_static_plan_bag_never_infers_candidates_from_consumer_or_fake_fields() {
        for source in [
            "정수:바른수 := 1. 근사:셈수 := 1. 가방 <- (0,차림[정수,근사]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값+1) 보여주기.",
            "가방 <- (0,차림[1]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. 결과:셈수 := 뽑음.값.",
            "가방 <- (0,차림[1]) 무작위가방.만들기. { 거짓 }이면 { 가방 <- (0,차림[0.5]) 무작위가방.만들기. }. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값+1) 보여주기.",
            "채비 { 가방:무작위가방. }. 뽑음 <- (가방) 무작위가방.꺼내기. (뽑음.값+1) 보여주기.",
            "뽑음 <- (값:1,가방:없음). (뽑음.값+1) 보여주기.",
            "가방 <- (0,차림[1]) 무작위가방.만들기. 뽑음 <- (가방) 무작위가방.꺼내기. 근사:셈수 := 1. (뽑음.값==근사) 보여주기.",
        ] {
            assert!(DdnProgram::from_v25_source(source,"bag-no-forged-kind.ddn").is_err(),"{source}");
        }
        // General legacy operation is not banned just because these candidates
        // cannot supply one numeric element type to a later consumer.
        let source="정수:바른수 := 1. 근사:셈수 := 1. 가방 <- (0,차림[정수,근사]) 무작위가방.만들기. 가방 보여주기.";
        crate::runtime_surface::run_summary_from_supported_source_profile(source,"bag-general-legacy.ddn",Some(1),Some("v1-core-v25")).unwrap();
    }

    #[test]
    fn b01_static_plan_selection_uses_all_source_candidates_without_consumer_retyping() {
        for (source,expected) in [
            ("자료 <- 차림[1,2]. 위치 <- 1. (자료[위치]+1) 보여주기.",vec!["3"]),
            ("판 <- 차림[차림[1,2],차림[3,4]]. 행 <- 1. 열 <- 0. (판[행][열]+1) 보여주기.",vec!["4"]),
            ("자료 <- 차림[1,2,3]. 처음 <- 1. 끝 <- 3. 조각 <- 자료[처음..끝]. 위치 <- 0. (조각[위치]+1) 보여주기.",vec!["3"]),
            ("자료 <- 차림[차림[1,2],차림[3,4]]. 위치 <- 0. 선택 <- 자료[위치]. 선택 <- 자료[위치]. (선택[1]+1) 보여주기.",vec!["3"]),
            ("(자료:(바른수)차림) 늘림:(바른수)차림:셈씨 := { 결과 <- (자료,3) 추가. 늘림 <- 결과. }. 값 <- (자료=차림[1]) 늘림. 위치 <- 1. (값[위치]+1) 보여주기.",vec!["4"]),
        ] {
            let result=crate::runtime_surface::run_summary_from_supported_source_profile(source,"n1-selection-source.ddn",Some(1),Some("v1-core-v25")).unwrap_or_else(|error|panic!("{source}: {error}"));
            assert_eq!(result["output_display"]["texts"],serde_json::json!(expected),"{source}");
        }
        let mixed="자료 <- 차림[1,0.5]. 위치 <- 0. 값:셈수 := 자료[위치].";
        assert!(DdnProgram::from_v25_source(mixed,"n1-no-consumer-retag.ddn").is_err(),"an expected result must not retype the source candidates");
        let mixed_nested="자료 <- 차림[차림[1,2],차림[0.5,1.5]]. 위치 <- 0. 선택 <- 자료[위치]. 선택 <- 자료[위치].";
        assert!(DdnProgram::from_v25_source(mixed_nested,"n1-no-nested-consumer-retag.ddn").is_err());
    }

    #[test]
    fn b01_static_plan_registered_unit_owner_keeps_dimensioned_overloads() {
        for (source,expected) in [
            ("면적 <- 2@m * 3@m. 면적 보여주기.",vec!["6@m^2"]),
            ("각 <- 0@rad. (각) sin 보여주기.",vec!["0"]),
            ("면적 <- 4@m * 4@m. (면적) sqrt 보여주기.",vec!["4@m"]),
            ("비율 <- 1@m / 1@m. 비율 보여주기.",vec!["1"]),
        ] {
            let result=crate::runtime_surface::run_summary_from_supported_source_profile(source,"n1-unit-owner.ddn",Some(1),Some("v1-core-v25")).unwrap_or_else(|error|panic!("{source}: {error}"));
            assert_eq!(result["output_display"]["texts"],serde_json::json!(expected),"{source}");
        }
        for source in ["(1@m) sin 보여주기.","(1@m) sqrt 보여주기.","값:나눔수 := (0@rad) sin."] {
            assert!(crate::runtime_surface::run_summary_from_supported_source_profile(source,"n1-unit-wrong.ddn",Some(1),Some("v1-core-v25")).is_err(),"{source}");
        }
    }

    #[test]
    fn b01_static_plan_common_frontdoor_issues_before_execution() {
        for (source, expected) in [
            ("값:나눔수 := 0.1+0.2. (값 == (3/10)) 보여주기.", vec!["참"]),
            ("분자값 <- 3. 분모값 <- 10. (분자값/분모값) 보여주기.", vec!["3/10"]),
            ("시간 <- 0. 간격 <- 0.02. 시간 <- 시간+간격. 시간 보여주기.", vec!["0.02"]),
        ] {
            let program = DdnProgram::from_v25_source(source,"numeric-frontdoor.ddn").unwrap();
            assert!(program.numeric_constraint_plan.is_some());
            let summary = crate::runtime_surface::run_summary_from_supported_source_profile(source,"numeric-frontdoor.ddn",Some(1),Some("v1-core-v25")).unwrap();
            assert_eq!(summary["output_display"]["texts"],serde_json::json!(expected),"{source}");
        }
    }

    #[test]
    fn b01_static_plan_declared_su_and_scalar_math_share_owner() {
        for (source,expected) in [
            ("값:수 := 0.1. 값 보여주기.",vec!["0.1"]),
            ("값:수 := 1. 값 보여주기.",vec!["1"]),
            ("(0) sin 보여주기. (0) cos 보여주기. (4) sqrt 보여주기.",vec!["0","1","2"]),
            ("(값:나눔수) sin:나눔수:셈씨 := { sin <- 값. }. 정확:나눔수 := 0.1. (정확) sin 보여주기.",vec!["1/10"]),
            ("(값:나눔수) 제곱근:나눔수:셈씨 := { 제곱근 <- 값. }. 정확:나눔수 := 0.1. (정확) 제곱근 보여주기.",vec!["1/10"]),
        ] {
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"scalar-owner.ddn",Some(1),Some("v1-core-v25"))
                .unwrap_or_else(|error|panic!("{source}: {error}"));
            assert_eq!(summary["output_display"]["texts"],serde_json::json!(expected),"{source}");
        }
        for source in [
            "값:나눔수 := 0.1. 99 보여주기. (값) sin 보여주기.",
            "값:바른수 := 4. 99 보여주기. (값) sqrt 보여주기.",
            "(0,1) cos 보여주기.",
        ] {
            let error=DdnProgram::from_v25_source(source,"scalar-rejected.ddn").err().expect(source);
            assert!(error.to_string().contains("E_TYPE_MISMATCH"),"{source}: {error}");
        }
        let domain="(-1) sqrt 보여주기.";
        let program=DdnProgram::from_v25_source(domain,"scalar-domain.ddn").unwrap();
        let error=DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).err().unwrap();
        assert!(error.to_string().contains("음수의 제곱근"),"{error}");
    }

    #[test]
    fn b01_static_plan_coarse_collection_uses_source_call_and_return_positions() {
        for (source,expected) in [
            ("(자료:차림) 첫합:수:셈씨 := { 첫합 <- 자료[0]+자료[1]. }. ((차림[0.5,1])) 첫합 보여주기.","1.5"),
            ("(자료:차림) 전달:차림:셈씨 := { 전달 <- 차림[자료[0],자료[1]]. }. (자료:차림) 계산:수:셈씨 := { 중간 := (자료) 전달. 계산 <- 중간[0]+중간[1]. }. ((차림[0.5,1])) 계산 보여주기.","1.5"),
        ] {
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"collection-call-owner.ddn",Some(1),Some("v1-core-v25"))
                .unwrap_or_else(|error|panic!("{source}: {error}"));
            assert_eq!(summary["output_display"]["texts"],serde_json::json!([expected]));
        }
        let source="(자료:차림) 계산:수:셈씨 := { 계산 <- 자료[0]+자료[1]. }. 정수값:바른수 := 1. ((차림[0.5,정수값])) 계산 보여주기.";
        assert!(DdnProgram::from_v25_source(source,"collection-issued-conflict.ddn").is_err());
        let wrong="(값:수~의) 두배:수:셈씨 := { 두배 <- 값*2. }. 실행:움직씨 := { 99 보여주기. \"가\"의 두배 보여주기. }. () 실행하기.";
        let error=DdnProgram::from_v25_source(wrong,"course-08A-static.ddn").err().unwrap();
        assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"),"{error}");
        assert!(error.contains("실행 전 원문"),"{error}");
    }

    #[test]
    fn b01_static_plan_atan2_result_requires_independent_scalar_owners() {
        use crate::numeric_constraint_plan::NumericKind;
        for (source,expected) in [
            ("(0.0,1.0) atan2 보여주기.","0"),
            ("(1.0,0.0) atan2 보여주기.","1.570796"),
            ("(0.0,-1.0) atan2 보여주기.","3.141593"),
            ("(0.0,0.0) atan2 보여주기.","0"),
        ] {
            let program=DdnProgram::from_v25_source(source,"atan2-positive.ddn").unwrap();
            let Stmt::Show{expr,..}=&program.program.root_stmts[0] else {panic!("show");};
            let plan=program.numeric_constraint_plan.as_ref().unwrap();
            assert_eq!(plan.expressions.get(&expr.id),Some(&TypeRef::Named("셈수".into())));
            assert!(plan.literals.values().all(|kind|*kind==NumericKind::Approximate));
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"atan2-positive.ddn",Some(1),Some("v1-core-v25")).unwrap();
            assert_eq!(summary["output_display"]["texts"],serde_json::json!([expected]));
        }
        for source in [
            "(0,1) atan2 보여주기.",
            "값:바른수 := 1. (값,값) atan2 보여주기.",
            "값:나눔수 := 0.5. (값,값) atan2 보여주기.",
            "(1.0@rad,1.0@rad) atan2 보여주기.",
        ] {
            let program=DdnProgram::from_v25_source(source,"atan2-no-supplier.ddn").unwrap();
            let expr=program.program.root_stmts.iter().find_map(|statement|match statement {Stmt::Show{expr,..}=>Some(expr),_=>None}).unwrap();
            let plan=program.numeric_constraint_plan.as_ref().unwrap();
            assert!(plan.expressions.get(&expr.id).is_none(),"no atan2 result supplier: {source}");
            if source=="(0,1) atan2 보여주기." {assert!(plan.literals.values().all(|kind|*kind==NumericKind::Integer));}
        }
        let shadow="(y:나눔수,x:나눔수) atan2:나눔수:셈씨 := { atan2 <- y+x. }. (0.1,0.2) atan2 보여주기.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(shadow,"atan2-user.ddn",Some(1),Some("v1-core-v25")).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["3/10"]));
        for source in ["99 보여주기. ((0,1) atan2)+0.1 보여주기.","99 보여주기. (0.0) atan2 보여주기."] {
            assert!(DdnProgram::from_v25_source(source,"atan2-atomic-reject.ddn").is_err(),"{source}");
        }
        assert!(DdnProgram::from_v25_source("값 <- 0. 값 <- (값,값) atan2. 값 보여주기.","atan2-self-certification.ddn").is_err(),"atan2 output must not certify integer-default pins");
    }

    #[test]
    fn b01_static_plan_tuple_world_uses_same_position_owners() {
        let source="묶음값 <- (1,0.5,참). 묶음값[1] 보여주기.";
        let program=DdnProgram::from_v25_source(source,"numeric-tuple.ddn").unwrap();
        let runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let expected=runner.tuple_type_plan.expected("묶음값").expect("same static positional supplier");
        assert_eq!(expected,&ddonirang_core::platform::RecordFieldType::Tuple{items:["바른수","셈수","참거짓"].into_iter()
            .map(|name|ddonirang_core::platform::RecordFieldType::Named{name:name.into()}).collect()});
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"numeric-tuple.ddn",Some(1),Some("v1-core-v25")).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["0.5"]));
        let rejected="원값:바른수 := 1. 묶음값 <- (원값,참). 묶음값[0] <- 0.5.";
        assert!(DdnProgram::from_v25_source(rejected,"numeric-tuple-wrong.ddn").is_err());
    }

    #[test]
    fn b01_static_plan_rational_tuple_uses_approved_genuine_carrier() {
        let local="묶음값 := (1,1/3,참). 묶음값[1] 보여주기.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(local,"numeric-tuple-local.ddn",Some(1),Some("v1-core-v25")).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["1/3"]));
        // S tag10 now supplies the numeric owner, not a Map/Pack shape.
        let stored="묶음값 <- (1,1/3,참). 묶음값[1] 보여주기.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(stored,"numeric-tuple-storage.ddn",Some(1),Some("v1-core-v25")).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["1/3"]));
        let numeric=make_rational_pack_from_big_rational(&BigRational::new(1.into(),3.into()));
        let kind=ddonirang_core::platform::RecordFieldType::Named{name:"나눔수".into()};
        assert!(matches!(resource_value_from_typed_value(&numeric,&kind).unwrap(),ResourceValue::RationalV1(_)));
        assert!(resource_value_from_value(&numeric).is_err(),"no source-less persistence");
        let fake=make_numeric_pack(NUMERIC_KIND_RATIONAL,BTreeMap::new());
        assert_eq!(numeric_pack_kind(&fake),None);
        assert!(resource_value_from_typed_value(&fake,&kind).is_err(),"shape is not a numeric owner");
    }

    #[test]
    fn b01_static_plan_chaebi_and_hook_share_authored_global_owner() {
        let source="채비 { 시간 := 0. 간격 := 0.02. }. (매마디)마다 { 시간 <- 시간+간격. 시간 보여주기. }.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"numeric-chaebi.ddn",Some(1),Some("v1-core-v25")).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["0.02"]));
    }

    #[test]
    fn b01_static_plan_declared_field_and_element_execute_through_frontdoor() {
        for (source, expected) in [
            ("(가로:셈수, 세로:나눔수) 좌표:묶음씨 := {}. (점:좌표) 계산:셈수:셈씨 := { 계산 <- 점.가로 + 1. }. (점=(가로=0.5,세로=1/3) 좌표) 계산 보여주기.", "1.5"),
            ("(값들:(셈수)차림, 번호:바른수) 계산:셈수:셈씨 := { 계산 <- 값들[번호] + 1. }. (값들=차림[0.5,1.5],번호=1) 계산 보여주기.", "2.5"),
        ] {
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"numeric-owned-fields.ddn",Some(1),Some("v1-core-v25")).unwrap();
            assert_eq!(summary["output_display"]["texts"],serde_json::json!([expected]),"{source}");
        }
    }

    #[test]
    fn b01_static_plan_conflict_is_admission_failure_not_partial_output() {
        let source = "(입력값:셈수) 근사값:셈수:셈씨 := { 근사값 <- 입력값. }. (입력값:나눔수) 정확값:나눔수:셈씨 := { 정확값 <- 입력값. }. 변수 <- 1. 9 보여주기. (변수) 근사값 보여주기. (변수) 정확값 보여주기.";
        let error = DdnProgram::from_v25_source(source,"numeric-conflict.ddn").err().expect("conflict before runner exists");
        assert!(error.contains("서로 다른 씨"),"{error}");
        assert!(crate::runtime_surface::run_summary_from_supported_source_profile(source,"numeric-conflict.ddn",Some(1),Some("v1-core-v25")).is_err());
    }

    #[test]
    fn b01_original_direct_comparison_uses_static_rational_expectation() {
        let mut positives = vec![("값:나눔수 := 0.1 + 0.2.\n(값 == (3/10)) 보여주기.\n".to_string(),vec!["참"])];
        for op in ["==", "!=", "<", "<=", ">", ">="] {
            for (left,right) in [("값","(3/10)"),("(3/10)","값"),("(-값)","(-(3/10))")] {
                positives.push((format!("값:나눔수 := 0.1+0.2. 9 보여주기. ({left} {op} {right}) 보여주기."),
                    vec!["9",if matches!(op,"=="|"<="|">="){"참"}else{"거짓"}]));
            }
        }
        for source in [
            "값:나눔수 := 0.1+0.2. 복사 <- 3/10. (값 == 복사) 보여주기.",
            "값:나눔수 := 0.1+0.2. ((값,참) == (3/10,참)) 보여주기.",
            "값:나눔수 := 0.1+0.2. (차림[값] == 차림[3/10]) 보여주기.",
            "(((3,10) 나눔수) == (3/10)) 보여주기.",
        ] {
            // The approved N1 integer-division/default supplier now issues
            // these exact owners before evaluation, including nested values.
            positives.push((source.to_string(),vec!["참"]));
        }
        for source in [
            "(분자값:수,분모값:수) 몫:수:셈씨 := { 몫 <- 분자값/분모값. }. 값:나눔수 := 0.1+0.2. (값 == (분자값=3,분모값=10) 몫) 보여주기.",
        ] {
            let canon=ddonirang_lang::parse_v25_and_normalize(source,"original-direct.ddn",ddonirang_lang::NormalizationLevel::N1)
                .unwrap_or_else(|error|panic!("{source}: {error}"));
            for text in [source,canon.as_str()] {
                let world=NuriWorld::new(); let before=world.state_hash();
                let error=DdnProgram::from_v25_source(text,"original-direct.ddn")
                    .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                        .run_update(&world,&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                    .err().expect(text);
                assert!(error.contains("E_NUMERIC_COMPARISON_TYPE_MISMATCH"),"{text}: {error}");
                assert!(error.contains("자동 변환"),"{error}");
                assert_eq!(world.state_hash(),before);
            }
        }
        for source in [
            "값:나눔수 := 0.1+0.2. (값 == ((3/10)+0)) 보여주기.",
            "값:나눔수 := 0.1+0.2. (값 == (0.1+0.2)) 보여주기.",
            "값:나눔수 := 1. (값 == (2/2)) 보여주기.",
            "값:나눔수 := 0.1+0.2. 비교값:나눔수 := 3/10. (값 == 비교값) 보여주기.",
            "값:나눔수 := 0.1+0.2. (값 == ((3,10) 나눔수)) 보여주기.",
            "값:나눔수 := 0.5. (값 == 0.5) 보여주기.",
        ] {
            positives.push((source.to_string(),vec!["참"]));
        }
        // A separate uncontextualized integer division now has N1's rational
        // default; preserve its exact representation instead of Fixed64 text.
        positives.push(("((3/10) == 0.3) 보여주기. (3/10) 보여주기.".into(),vec!["참","3/10"]));
        for (source,expected) in positives {
            let canon=ddonirang_lang::parse_v25_and_normalize(&source,"original-positive.ddn",ddonirang_lang::NormalizationLevel::N1).unwrap();
            for text in [&source,&canon] {
                let summary=crate::runtime_surface::run_summary_from_supported_source_profile(text,"original-positive.ddn",Some(1),Some("v1-core-v25")).unwrap();
                assert_eq!(summary["output_display"]["texts"],serde_json::json!(expected),"{text}");
            }
        }
    }

    #[test]
    fn b01_typed_numeric_comparison_preserves_same_kind_and_pending_literals() {
        for source in [
            "왼값:나눔수 := 0.1. 오른값:나눔수 := 0.1. (왼값 == 오른값) 보여주기.",
            "왼값:셈수 := 1.0. 오른값:셈수 := 1.0. (왼값 == 오른값) 보여주기.",
            "(1 == 1.0) 보여주기. ((1/2) == 0.5) 보여주기.",
        ] {
            let program = DdnProgram::from_v25_source(source, "compare-compatible.ddn").unwrap();
            DdnRunner::new(program, DEFAULT_UPDATE_NAME)
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        }
    }

    #[test]
    fn b01_pending_literal_mixing_preserves_existing_scalar_execution() {
        let source="채비 { 실수:셈수 := 1. }. 혼합 <- 1+0.2. 실수 보여주기. 혼합 보여주기.";
        let output=DdnRunner::new(DdnProgram::from_v25_source(source,"pending-mixed.ddn").unwrap(),DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["실수"],Value::Fixed64(Fixed64::from_i64(1)));
        assert_eq!(output.resources["혼합"],Value::Fixed64(Fixed64::from_i64(1)+Fixed64::parse_decimal_exact("0.2").unwrap()));
    }

    #[test]
    fn vc13_tuple_declared_callable_results_propagate_without_literal_defaults() {
        let source = r#"
            (원값:바른수) 정수전달:바른수:셈씨 := { 정수전달 <- 원값. }.
            (원값:셈수) 실수전달:셈수:셈씨 := { 실수전달 <- 원값. }.
            정수:바른수 := 7. 실수:셈수 := 7. 글값:글 := "가".
            정수묶음 <- ((원값=정수) 정수전달,글값).
            소수묶음 <- ((원값=실수) 실수전달,글값).
            복사묶음 <- 정수묶음[0..2].
            정수묶음[0] <- (원값=정수) 정수전달.
            정수묶음[0] 보여주기.
        "#;
        let program = DdnProgram::from_v25_source(source, "tuple-declared-call.ddn").unwrap();
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let mut world = DetNuri::new();
        world.apply_patch(&output.patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
        let a = world.world().get_resource_value("정수묶음").unwrap();
        let b = world.world().get_resource_value("소수묶음").unwrap();
        let (ResourceValue::Tuple { element_types: a_types, items: a_items }, ResourceValue::Tuple { element_types: b_types, items: b_items }) = (&a, &b) else { panic!("typed call Tuple"); };
        assert_eq!(a_items, b_items);
        assert_ne!(a_types, b_types);
        assert_eq!(a_types[0], ddonirang_core::platform::RecordFieldType::Named { name: "바른수".into() });
        assert_eq!(b_types[0], ddonirang_core::platform::RecordFieldType::Named { name: "셈수".into() });
        let bad = source.replace("정수묶음[0] <- (원값=정수) 정수전달", "정수묶음[0] <- (원값=실수) 실수전달");
        let before = world.world().state_hash();
        let error=DdnProgram::from_v25_source(&bad, "tuple-wrong-call.ddn")
            .and_then(|program|DdnRunner::new(program, DEFAULT_UPDATE_NAME)
                .run_update(world.world(), &empty_input(), &HashMap::new()).map_err(|error|error.to_string()))
            .err().expect("known incompatible callable result must not be published");
        assert!(error.contains("E_TUPLE_REPLACEMENT"),"{error}");
        assert_eq!(world.world().state_hash(), before);
        // Owner-approved literal source defaults now supply the ordered
        // child identities; the earlier no-supplier expectation is obsolete.
        let program = DdnProgram::from_v25_source("값 <- (1,\"글\",참). 값[0] 보여주기.", "b01-open.ddn").unwrap();
        let plan = crate::tuple_saved_type_plan::TupleSavedTypePlan::compile(
            &program.program, Some(DEFAULT_UPDATE_NAME), true,
            &program.top_level_decl_names, &program.record_schemas, program.numeric_constraint_plan.as_deref(),&program.nominal_type_ids());
        assert_eq!(plan.expected("값"), Some(&ddonirang_core::platform::RecordFieldType::Tuple {
            items: ["바른수", "글", "참거짓"].into_iter().map(|name|
                ddonirang_core::platform::RecordFieldType::Named { name: name.into() }).collect(),
        }));
    }

    #[test]
    fn vc13_tuple_source_declared_types_reach_world_without_shape_inference() {
        // A separately authored runnable product example, not a lowering of
        // the design fixture (whose standalone `가` is NAME-LINT-01).
        let source = "채비 { 정수:바른수 := 7. 실수:셈수 := 7. 글값:글 := \"가\". 논값:참거짓 := 참. 정수묶음 := (정수,글값,논값). 소수묶음 := (실수,글값,논값). 빈묶음 := (). 중첩묶음 := (정수묶음,소수묶음). 범위묶음 := 정수묶음[0..2]. 선택묶음 := (정수묶음[1],논값). }. 정수묶음 <- (정수,글값,논값).";
        let program = DdnProgram::from_v25_source(source, "tuple-source.ddn").unwrap();
        let plan = crate::tuple_saved_type_plan::TupleSavedTypePlan::compile(&program.program, Some(DEFAULT_UPDATE_NAME), true, &program.top_level_decl_names, &program.record_schemas, program.numeric_constraint_plan.as_deref(),&program.nominal_type_ids());
        assert!(plan.expected("정수묶음").is_some(), "{plan:#?}; functions={:?}; declarations={:?}", program.functions.keys(), program.top_level_decl_names);
        let mut runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let mut world = DetNuri::new();
        world.apply_patch(&output.patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
        let a = world.world().get_resource_value("정수묶음").unwrap();
        let b = world.world().get_resource_value("소수묶음").unwrap();
        let (ResourceValue::Tuple { element_types: a_types, items: a_items }, ResourceValue::Tuple { element_types: b_types, items: b_items }) = (&a, &b) else { panic!("typed tuples"); };
        assert_eq!(a_items, b_items);
        assert_ne!(a_types, b_types);
        let mut a_bytes = Vec::new(); a.encode_canon(&mut a_bytes);
        let mut b_bytes = Vec::new(); b.encode_canon(&mut b_bytes);
        assert_ne!(a_bytes, b_bytes);
        for name in ["빈묶음", "중첩묶음", "범위묶음", "선택묶음"] { assert!(matches!(world.world().get_resource_value(name), Some(ResourceValue::Tuple { .. })), "{name}"); }
    }

    #[test]
    fn vc13_tuple_saved_project_restore_requires_same_source_type_and_is_atomic() {
        use crate::authored_project::{ProjectArtifact, SourceSnapshot, SourceFile};
        let root = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body = "정수:바른수 := 7. 실수:셈수 := 7. 글값:글 := \"가\". 논값:참거짓 := 참. 정수묶음 := (정수,글값,논값). 소수묶음 := (실수,글값,논값). 실행:움직씨 := { 정수묶음[0] <- 정수. 정수묶음[1] 보여주기. }.\n";
        let artifact = ProjectArtifact::create("v1-core-v25".into(), SourceSnapshot {
            root_path: Some("root.ddn".into()), entry_path: "app/main.ddn".into(), files: vec![
                SourceFile { path: "root.ddn".into(), text: root.into() },
                SourceFile { path: "app/main.ddn".into(), text: body.into() },
            ],
        }, String::new()).unwrap();
        let saved = artifact.save().unwrap();
        let owned = crate::project_revision_envelope_supply::admit_fresh_authored_project(saved.as_bytes()).unwrap();
        let make_runner = || {
            let program = DdnProgram::from_saved_v25_project_with_mode(owned.sources().clone(), ParseMode::Strict, AgeTarget::Age3).unwrap();
            let mut runner = DdnRunner::new(program, "실행");
            runner.configure_project_entry("실행").unwrap(); runner
        };
        let mut runner = make_runner();
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let mut world = DetNuri::new();
        world.apply_patch(&output.patch, 0, &mut ddonirang_core::signals::VecSignalSink::default());
        let snapshot = crate::runtime_surface::runtime_state_snapshot(&world, Vec::new(), 1)["resources"].clone();
        let raw = crate::unit_wire::serialize_resource_snapshot_json(&snapshot);
        let parsed = crate::unit_wire::parse_snapshot_json(&raw).unwrap();
        // No source owner is explicit rejection, even for valid bytes/hashes.
        let mut restored = NuriWorld::new();
        restored.set_resource_value("보존".into(), ResourceValue::Bool(true));
        let before = restored.state_hash();
        assert!(crate::unit_wire::restore_world_resources_snapshot(&mut restored, &parsed).is_err());
        assert_eq!(restored.state_hash(), before);
        // A fresh compiler re-resolves the saved closure and exact entry.
        let mut reopened = make_runner();
        let plan = reopened.saved_tuple_restore_plan().unwrap();
        crate::unit_wire::restore_world_resources_with_tuple_plan(&mut restored, &parsed, Some(&plan)).unwrap();
        assert_eq!(restored.state_hash(), world.world().state_hash());
        let baseline = runner.run_update(world.world(), &empty_input(), &HashMap::new()).unwrap();
        let resumed = reopened.run_update(&restored, &empty_input(), &HashMap::new()).unwrap();
        assert_eq!(baseline.patch.ops, resumed.patch.ops);
        // Valid foreign bytes + recomputed length/hash are not type custody:
        // same raw values, declared element0 differs (바른수 versus 셈수).
        let mut forged = snapshot.clone();
        forged["value_det"]["정수묶음"] = snapshot["value_det"]["소수묶음"].clone();
        let before = restored.state_hash();
        assert!(crate::unit_wire::restore_world_resources_with_tuple_plan(&mut restored, &forged, Some(&plan)).is_err());
        assert_eq!(restored.state_hash(), before);
    }

    #[test]
    fn vc13_tuple_missing_or_conflicting_source_evidence_rejects_without_a_patch() {
        let changed = DdnProgram::from_v25_source("채비 { 처음값:바른수 := 0. 바꿀값:바른수 := 8. 상태묶음 := (처음값,). }. 상태묶음[0] <- 바꿀값.", "tuple-change.ddn").unwrap();
        let changed = DdnRunner::new(changed, DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let last = changed.patch.ops.iter().rev().find_map(|op| match op {
            PatchOp::SetResourceValue { tag, value: ResourceValue::Tuple { items, .. } } if tag == "상태묶음" => Some(items), _ => None,
        }).unwrap();
        assert_eq!(last, &vec![ResourceValue::Fixed64(Fixed64::from_i64(8))]);
        for (source,code) in [
            // The literal case now has an approved source supplier. A sealed
            // builtin result still cannot borrow an ordinary declaration's
            // type, so retain the missing-evidence negative on that boundary.
            ("묶음값 <- ((4) abs, \"가\").","E_TUPLE_PERSISTENCE_UNBOUND"),
            ("채비 { 원정수:바른수 := 7. 원수:셈수 := 7. 글값:글 := \"가\". 묶음값 := (원정수,글값). }. 묶음값 <- (원수,글값).","E_TYPE_MISMATCH"),
        ] {
            let world = NuriWorld::new(); let before = world.state_hash();
            let error=DdnProgram::from_v25_source(source, "tuple-unsupported.ddn")
                .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME)
                    .run_update(&world,&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
                .err().expect("unsupported or conflicting owner must reject");
            assert!(error.contains(code), "{error}");
            assert_eq!(world.state_hash(), before);
        }
        // Existing Named 수 is not a forbidden alias. Source provenance, not
        // the limited reference Python's whitelist, owns the declared type.
        let program = DdnProgram::from_v25_source("채비 { 입력수:수 := 7. 묶음값 := (입력수,). }.", "tuple-su.ddn").unwrap();
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        assert!(output.patch.ops.iter().any(|op| matches!(op, PatchOp::SetResourceValue { tag, value: ResourceValue::Tuple { element_types, .. } }
            if tag == "묶음값" && element_types == &vec![ddonirang_core::platform::RecordFieldType::Named { name: "셈수".into() }])));
    }

    #[test]
    fn vc13_tuple_constant_arithmetic_selection_preserves_positions_and_alias() {
        // RD-352 requires compile-time static positions, not literal-only
        // positions. This uses the unchanged raw source frontdoor.
        let source = r#"검사:참거짓:셈씨 := {
            값 <- (1, "글", 참). 이전 <- 값.
            값[(1 + 0)] <- "바뀜".
            검사 <- 값[(2 / 2)] == "바뀜"
                그리고 이전[(1.5 - 0.5)] == "글"
                그리고 값[(0.5 * 4)] == 참.
        }. 결과 <- () 검사."#;
        let analysis = crate::source_analysis::analyze_source(crate::source_analysis::SourceAnalysisRequest {
            source_text: source.into(), path: "tuple-constant-expression.ddn".into(),
            analysis_generation: 0, source_profile_identity: V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY.into(),
        });
        assert!(analysis.diagnostics.is_empty(), "{:?}", analysis.diagnostics);
        let program = DdnProgram::from_v25_source(source, "tuple-constant-expression.ddn").unwrap();
        let world = NuriWorld::new();
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME)
            .run_update(&world, &empty_input(), &HashMap::new()).unwrap();
        assert_eq!(output.resources["결과"], Value::Bool(true));
        let saved_source = "채비 { 정수:바른수 := 7. 글값:글 := \"가\". 위치값 := (정수,글값). 선택값 := (위치값[(1 - 1)],글값). }. 위치값[(2 / 2)] <- 글값.";
        let program = DdnProgram::from_v25_source(saved_source, "tuple-static-source-type.ddn").unwrap();
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME)
            .run_update(&world, &empty_input(), &HashMap::new()).unwrap();
        assert!(output.patch.ops.iter().any(|op| matches!(op,
            PatchOp::SetResourceValue { tag, value:ResourceValue::Tuple { element_types, .. } }
                if tag=="선택값" && element_types==&vec![
                    ddonirang_core::platform::RecordFieldType::Named { name:"바른수".into() },
                    ddonirang_core::platform::RecordFieldType::Named { name:"글".into() },
                ])));
    }

    #[test]
    fn vc13_tuple_rejects_dynamic_wrong_type_and_unbound_persistence() {
        for (body,code) in [
            ("위치 <- 0. 값 <- (1, \"글\"). 검사 <- 값[위치].","E_CHECK_TUPLE_SELECTION"),
            ("값 <- (1, \"글\"). 값[0] <- \"잘못된씨\". 검사 <- 1.","E_TUPLE_REPLACEMENT"),
            ("값 <- (1, \"글\"). 값[0..2] <- (2, 참). 검사 <- 1.","E_TUPLE_REPLACEMENT"),
            ("값 <- (1, \"글\"). 검사 <- 값[2].","E_CHECK_TUPLE_SELECTION"),
        ] {
            let source=format!("검사:수:셈씨 := {{ {body} }}. 결과 <- () 검사.");
            let world=NuriWorld::new();let before=world.state_hash();
            let error=DdnProgram::from_v25_source(&source,"vc13-error.ddn").and_then(|program|
                DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&world,&empty_input(),&HashMap::new()))
                .err().expect("invalid selection must fail before output publication");
            assert!(error.contains(code),"{body}: {error}");
            assert_eq!(world.state_hash(),before);
        }
        let value=Value::Tuple(vec![Value::Bool(true)]);
        assert!(resource_value_from_value(&value).err().unwrap().to_string().contains("E_TUPLE_PERSISTENCE_UNBOUND"));
    }

    #[test]
    fn beta1_text_bool_resources_use_typed_committed_values() {
        let program=DdnProgram::from_v25_source("글값 <- \"참\". 참값 <- 참. 거짓값 <- 거짓.","글참거짓.ddn").unwrap();
        let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output=runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert!(output.patch.ops.iter().all(|op| !matches!(op,PatchOp::SetResourceJson {..})));
        let mut world=DetNuri::new();
        world.apply_patch(&output.patch,0,&mut ddonirang_core::signals::VecSignalSink::default());
        assert!(world.world().resource_json_entries().is_empty());
        assert_eq!(world.world().get_resource_value("글값"),Some(ResourceValue::String("참".into())));
        assert_eq!(world.world().get_resource_value("참값"),Some(ResourceValue::Bool(true)));
        assert_eq!(world.world().get_resource_value("거짓값"),Some(ResourceValue::Bool(false)));
        let copy=DdnProgram::from_v25_source("글복사 <- 글값. 참복사 <- 참값.","복사.ddn").unwrap();
        let output=DdnRunner::new(copy,DEFAULT_UPDATE_NAME).run_update(world.world(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["글복사"],Value::String("참".into()));
        assert_eq!(output.resources["참복사"],Value::Bool(true));
    }

    #[test]
    fn beta1_typed_resource_admission_rejects_legacy_and_cross_lane_without_change() {
        let mut legacy=NuriWorld::new();legacy.set_resource_json("글값".into(),"참".into());
        let mut duplicate=NuriWorld::new();duplicate.set_resource_fixed64("글값".into(),Fixed64::from_i64(1));
        duplicate.set_resource_value("글값".into(),ResourceValue::String("참".into()));
        for world in [legacy,duplicate] {
            let before=format!("{world:?}");
            let program=DdnProgram::from_v25_source("복사 <- 글값.","거부.ddn").unwrap();
            let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
            assert!(runner.run_update(&world,&empty_input(),&HashMap::new()).err().unwrap().contains("E_RESOURCE_SNAPSHOT_INVALID"));
            assert_eq!(format!("{world:?}"),before);
        }
        let program=DdnProgram::from_v25_source("같은값 <- 1. 같은값 <- 참.","이중통로.ddn").unwrap();
        assert!(DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).is_err());
    }

    #[test]
    fn beta1_typed_bool_restore_rejects_text_substitution_for_declared_binding() {
        let program=DdnProgram::from_source("채비 { 고정:참거짓 <- 거짓. }. 고정 보여주기.","형결속.ddn").unwrap();
        let runner=DdnRunner::new(program,"매틱");
        let mut world=NuriWorld::new();world.set_resource_value("고정".into(),ResourceValue::String("참".into()));
        let before=format!("{world:?}");
        assert!(runner.validate_persisted_records(&world).is_err());
        assert_eq!(format!("{world:?}"),before);
        world.set_resource_value("고정".into(),ResourceValue::Bool(true));
        runner.validate_persisted_records(&world).unwrap();
    }

    #[test]
    fn beta1_reference_admitted_world_tuple_survives_actual_source_copy_and_patch(){
        let saved=ResourceValue::IdentityReference(ddonirang_core::platform::IdentityReferenceValueV1{
            profile_version:1,target_type_identity:vec![0x11,0],owner_space_identity:vec![2;32],target_identity:vec![3;32],generation:0,
        }.into());
        let runtime=value_from_resource_value(&saved);assert_eq!(resource_value_from_value(&runtime).unwrap(),saved);
        let mut world=NuriWorld::new();world.set_resource_value("원래".into(),saved.clone());
        let program=DdnProgram::from_v25_source("복사 <- 원래.","참조값.ddn").unwrap();
        let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output=runner.run_update(&world,&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["복사"],runtime);
        assert!(output.patch.ops.iter().any(|op|matches!(op,PatchOp::SetResourceValue{tag,value} if tag=="복사"&&value==&saved)));
        assert_eq!(world.get_resource_value("원래"),Some(saved));
    }

    #[test]
    fn beta1_atom_admitted_world_value_survives_actual_source_copy_and_patch() {
        let (_,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"owner",&[vec![1]]).unwrap();
        let saved=ResourceValue::RegisteredAtom(contract.admitted_value(b"owner",&[1]).unwrap().into());
        let runtime=value_from_resource_value(&saved);
        assert_eq!(resource_value_from_value(&runtime).unwrap(),saved);
        let mut world=NuriWorld::new();world.set_resource_value("원래".into(),saved.clone());
        let program=DdnProgram::from_v25_source("복사 <- 원래.","등록값.ddn").unwrap();
        let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output=runner.run_update(&world,&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["복사"],runtime);
        assert!(output.patch.ops.iter().any(|op| matches!(op,PatchOp::SetResourceValue {tag,value} if tag=="복사" && value==&saved)));
        assert_eq!(world.get_resource_value("원래"),Some(saved));
    }

    #[test]
    fn vc04_atom_semantic_equality_is_consistent_inside_containers() {
        let (_, first) = crate::registered_atom_contract::RecordedAtomContract::record(
            b"owner", &[vec![1]],
        ).unwrap();
        let (_, extended) = crate::registered_atom_contract::RecordedAtomContract::record(
            b"owner", &[vec![1], vec![2]],
        ).unwrap();
        let left = ResourceValue::RegisteredAtom(first.admitted_value(b"owner", &[1]).unwrap().into());
        let right = ResourceValue::RegisteredAtom(extended.admitted_value(b"owner", &[1]).unwrap().into());
        // 저장 근거는 다르지만, 등록 owner와 원자의 의미 정체는 같다.
        assert_ne!(left, right);
        let mut world = NuriWorld::new();
        world.set_resource_value("원래".into(), left.clone());
        world.set_resource_value("다른출처".into(), right.clone());
        let before = world.state_hash();
        let source = "직접 <- 원래 == 다른출처. 목록 <- 차림[원래] == 차림[다른출처]. 묶음 <- (원래,) == (다른출처,).";
        let program = DdnProgram::from_v25_source(source, "등록원자비교.ddn").unwrap();
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME)
            .run_update(&world, &empty_input(), &HashMap::new()).unwrap();
        for name in ["직접", "목록", "묶음"] {
            assert_eq!(output.resources[name], Value::Bool(true), "{name}");
        }
        let left_value = value_from_resource_value(&left);
        let right_value = value_from_resource_value(&right);
        assert_eq!(left_value, right_value, "공유 Value 의미 비교");
        assert!(values_equal(
            &Value::Tuple(vec![left_value.clone()]),
            &Value::Tuple(vec![right_value.clone()]),
        ), "Tuple 보조 비교도 같은 원자 정체를 사용해야 한다");
        assert_eq!(Value::List(vec![left_value]), Value::List(vec![right_value]));
        let different_atom = value_from_resource_value(&ResourceValue::RegisteredAtom(
            extended.admitted_value(b"owner", &[2]).unwrap().into(),
        ));
        let (_, other_owner) = crate::registered_atom_contract::RecordedAtomContract::record(
            b"other-owner", &[vec![1]],
        ).unwrap();
        let different_owner = value_from_resource_value(&ResourceValue::RegisteredAtom(
            other_owner.admitted_value(b"other-owner", &[1]).unwrap().into(),
        ));
        for other in [different_atom, different_owner] {
            let original = value_from_resource_value(&left);
            assert!(!values_equal(&original, &other));
            assert_ne!(original.clone(), other.clone());
            assert!(!values_equal(&Value::Tuple(vec![original]), &Value::Tuple(vec![other])));
        }
        assert_eq!(world.get_resource_value("원래"), Some(left));
        assert_eq!(world.get_resource_value("다른출처"), Some(right));
        assert_eq!(world.state_hash(), before);
    }

    #[test]
    fn beta1_bytes_explicit_nfc_provenance_uses_the_same_byte_owner() {
        let source="보존 <- (글=\"가\") UTF8바이트열로. (바이트열=보존) UTF8NFC글로 보여주기.";
        let program=DdnProgram::from_v25_source(source,"바이트열.ddn").unwrap();
        let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output=runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        let bytes=ddonirang_core::bytes::ByteSequence::encode_utf8_exact("가");
        assert!(matches!(output.resources.get("보존"),Some(Value::Bytes(value)) if value==&bytes));
        assert_eq!(output.explicit_nfc_provenance.len(),1);
        let provenance=&output.explicit_nfc_provenance[0].1;
        assert_eq!(provenance.unicode_version,(17,0,0));
        assert_eq!(provenance.source_bytes_storage_sha256,bytes.storage_sha256());
        assert_eq!(provenance.decoded_text,"가");
    }

    #[test]
    fn alpha3_tensor_named_source_roundtrip_and_distinct_identity() {
        // RD-385/386/389/510/674/682/683, not tensor_v0 compatibility evidence.
        let source = r#"
벡터 <- (차림=차림[1,2],원소씨=셈수,모양=(2,)) 텐서로.
목록 <- (텐서=벡터) 차림으로.
영차원 <- (차림=7,원소씨=셈수,모양=()) 텐서로.
빈판 <- (차림=() 차림,원소씨=셈수,모양=(0,3)) 텐서로.
빈결과 <- (텐서=빈판) 차림으로.
행렬:(원소씨=셈수,모양=(2,2)) 텐서씨 := (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로.
확인 <- 행렬.
벡터 <- (차림=차림[3,4],원소씨=셈수,모양=(2,)) 텐서로.
"#;
        let program = DdnProgram::from_v25_source(source, "tensor.ddn").unwrap();
        let mut runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let RuntimeValue::Tensor(vector) = &output.resources["벡터"] else { panic!("typed Tensor"); };
        assert_eq!(vector.shape(), &[2]);
        assert_eq!(vector.elements(), &[Fixed64::from_i64(3),Fixed64::from_i64(4)]);
        let RuntimeValue::Tensor(scalar) = &output.resources["영차원"] else { panic!("rank 0"); };
        assert!(scalar.shape().is_empty());
        assert!(!values_equal(&output.resources["영차원"], &RuntimeValue::Fixed64(Fixed64::from_i64(7))));
        assert!(!values_equal(&output.resources["벡터"], &output.resources["목록"]));
        assert!(matches!(&output.resources["빈결과"], RuntimeValue::List(items) if items.is_empty()));
        let restored = crate::unit_wire::resource_value_from_detjson(
            &crate::unit_wire::resource_value_to_detjson(&crate::unit_wire::tensor_to_resource(vector).unwrap())).unwrap();
        assert_eq!(value_canon(&value_from_resource_value(&restored)), value_canon(&output.resources["벡터"]));
        assert!(matches!(&output.resources["확인"], RuntimeValue::Tensor(value) if value.shape() == [2,2]));
    }

    #[test]
    fn alpha3_tensor_source_single_axis_selection_and_persistent_update() {
        // RD-388/391: this is actual source, not harness lowering.
        let source = r#"
판 <- (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로.
옛판 <- 판.
둘째줄 <- 판[1].
마지막 <- 판[1][1].
판[1][0] <- 9.
바꾼값 <- 판[1][0].
옛값 <- 옛판[1][0].
"#;
        let program = DdnProgram::from_v25_source(source,"tensor-selection.ddn").unwrap();
        let mut runner = DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output = runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert!(matches!(&output.resources["둘째줄"],RuntimeValue::Tensor(t) if t.shape() == [2]));
        assert_eq!(output.resources["마지막"],RuntimeValue::Fixed64(Fixed64::from_i64(4)));
        assert_eq!(output.resources["바꾼값"],RuntimeValue::Fixed64(Fixed64::from_i64(9)));
        assert_eq!(output.resources["옛값"],RuntimeValue::Fixed64(Fixed64::from_i64(3)));
    }

    #[test]
    fn alpha3_tensor_multi_axis_source_slices_and_result_flow() {
        // RD-388/391: one bracket group is not nested list selection.
        let source = r#"
판 <- (차림=차림[차림[1,2,3],차림[4,5,6]],원소씨=셈수,모양=(2,3)) 텐서로.
옛판 <- 판.
마지막 <- 판[1,2].
가운데 <- 판[:,1..2].
빈판 <- 판[2..2,:].
새열 <- (차림=차림[차림[8],차림[9]],원소씨=셈수,모양=(2,1)) 텐서로.
판[:,1..2] <- 새열.
판[1,2] <- 7.
판[0..1,:][0][0] <- 10.
새열 -> 판[:,1..2].
뒤 <- 판[0,0].
이전 <- 옛판[0,0].
"#;
        let program = DdnProgram::from_v25_source(source,"tensor-multi.ddn").unwrap();
        let mut runner = DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let output = runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).unwrap();
        assert_eq!(output.resources["마지막"],RuntimeValue::Fixed64(Fixed64::from_i64(6)));
        assert_eq!(output.resources["뒤"],RuntimeValue::Fixed64(Fixed64::from_i64(10)));
        assert_eq!(output.resources["이전"],RuntimeValue::Fixed64(Fixed64::from_i64(1)));
        assert!(matches!(&output.resources["가운데"],RuntimeValue::Tensor(t) if t.shape() == [2,1] && t.elements() == [Fixed64::from_i64(2),Fixed64::from_i64(5)]));
        assert!(matches!(&output.resources["빈판"],RuntimeValue::Tensor(t) if t.shape() == [0,3] && t.elements().is_empty()));
        assert!(matches!(&output.resources["판"],RuntimeValue::Tensor(t) if t.elements() == [10,8,3,4,9,7].map(Fixed64::from_i64)));
    }

    #[test]
    fn alpha3_tensor_rejects_ragged_mismatched_element_and_replacement() {
        let prefix = "판 <- (차림=차림[1,2],원소씨=셈수,모양=(2,)) 텐서로.\n";
        for invalid in [
            "판 <- (차림=차림[1,2,3],원소씨=셈수,모양=(3,)) 텐서로.",
            "판 <- 차림[1,2].",
            "판 <- (차림=차림[1,\"가\"],원소씨=셈수,모양=(2,)) 텐서로.",
            "다른판 <- (차림=차림[차림[1,2],차림[3]],원소씨=셈수,모양=(2,2)) 텐서로.",
            "다른판:(원소씨=셈수,모양=(3,)) 텐서씨 := (차림=차림[1,2],원소씨=셈수,모양=(2,)) 텐서로.",
            "다른판 <- (텐서=차림[1,2]) 차림으로.",
            "판[2] <- 9.",
            "판[0.5] <- 9.",
            "값 <- 판[-1].",
            "값 <- 판[0.5].",
            "값 <- 판[:,0].",
            "값 <- 판[0,0].",
            "판[:] <- 9.",
            "판[:] <- (차림=차림[1],원소씨=셈수,모양=(1,)) 텐서로.",
        ] {
            let world = NuriWorld::new();
            let before = format!("{world:?}");
            // Known numeric conflicts are rejected before VM construction;
            // shape/bounds failures still exercise the runtime owner below.
            let outcome=DdnProgram::from_v25_source(&format!("{prefix}{invalid}"),"bad-tensor.ddn")
                .and_then(|program|DdnRunner::new(program,DEFAULT_UPDATE_NAME).run_update(&world,&empty_input(),&HashMap::new()));
            assert!(outcome.is_err(), "must reject: {invalid}");
            assert_eq!(format!("{world:?}"),before);
        }
    }

    #[test]
    fn pure_scalar_snapshot_rhs_whitelist_and_rollback() {
        let skipped = format!("값 <- 참 또는 ({}).", vec!["1"; 80].join(" + "));
        let skipped_program = DdnProgram::from_v25_source(&skipped, "skipped.ddn").unwrap();
        let Stmt::Mutate { value, .. } = &skipped_program.root_stmts[0] else { panic!("assignment"); };
        assert!(!expression_preserves_local_bindings(value), "deep skipped RHS must use full snapshot");
        for (statement, preserves) in [
            ("값 <- 2.", true),
            ("값 <- 값 + 1.", true),
            ("값 <- 1 / 0.", true),
            ("값 <- 참.", true),
            ("값 <- 차림[1, 2].", true),
            ("값 <- 차림[1, 1 / 0].", true),
            ("값 <- 차림[차림[값, 2], 차림[3]].", true),
            ("값 <- (첫째:1, 둘째:값).", true),
            ("값 <- 차림[(차림[1]) 길이].", false),
            ("값 <- (차림[1]) 길이.", false),
        ] {
            // Compile each RHS under an actual typed input owner. The private
            // frame assignment below is a defensive evaluator test, not a
            // claim that statically invalid assignments enter a public VM.
            let rhs=statement.strip_prefix("값 <- ").unwrap().strip_suffix('.').unwrap();
            let source = format!("(값:바른수) 계산:바른수:셈씨 := {{ 임시 := {rhs}. 계산 <- 1. }}.");
            let program = DdnProgram::from_v25_source(&source, "snapshot.ddn").unwrap();
            let seed=program.program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed)
                .find(|seed|seed.canonical_name=="계산").expect("declared test callable");
            let Stmt::Definition{id,span,mood,value,..}=&seed.body.as_ref().unwrap().stmts[0] else {panic!("RHS definition");};
            let statement_ast=Stmt::Mutate{id:*id,span:*span,mood:mood.clone(),
                target:Expr::new(*id,*span,ExprKind::Var("값".into())),value:value.clone(),deferred:false,deferred_span:None};
            let stmt=&statement_ast;
            let Stmt::Mutate { value, .. } = stmt else { panic!("assignment: {statement}: {stmt:?}"); };
            assert_eq!(expression_preserves_local_bindings(value), preserves);
            let world = NuriWorld::new();
            let defaults = HashMap::new();
            let mut context = EvalContext::new(&program, &world, &defaults, None,
                InputState::new(0, 0), 0, 0, None);
            context.current_seed_name = Some("계산".into());
            context.active_callable_name = Some("계산".into());
            assert!(context.owns_pure_local_bindings());
            let initial=make_big_int_pack_from_bigint(&BigInt::from(1));
            context.declare_binding_type("값", &inferred_binding_type(&initial));
            let mut locals = HashMap::from([
                ("값".into(), initial),
                ("판".into(), Value::List(vec![Value::List(vec![Value::Fixed64(Fixed64::ZERO); 10]); 20])),
            ]);
            let full = context.capture_frame_snapshot(&locals);
            let sparse = context.capture_frame_snapshot_with_locals(&locals, false);
            assert!(full.locals.is_some());
            assert!(sparse.locals.is_none());
            let before = format!("{:?}", locals);
            let evaluated = context.eval_expr(&mut locals, value);
            if preserves {
                assert_eq!(format!("{:?}", locals), before, "RHS must not write locals");
                context.restore_frame_snapshot(&mut locals, &sparse);
                assert_eq!(format!("{:?}", locals), before);
            }
            context.restore_frame_snapshot(&mut locals, &full);
            assert_eq!(format!("{:?}", locals), before);
            if statement == "값 <- 1 / 0." { assert!(evaluated.is_err()); }
            let result = context.eval_stmt(&mut locals, stmt);
            if !matches!(statement, "값 <- 2." | "값 <- 값 + 1."
                | "값 <- (차림[1]) 길이.") {
                assert!(result.is_err(), "failure must not publish: {statement}");
                assert_eq!(format!("{:?}", locals), before);
            } else {
                assert!(result.is_ok(), "expected successful assignment: {statement}");
            }
        }
    }

    #[test]
    fn local_snapshot_proof_is_bounded_and_checks_every_container_child() {
        for (source, expected) in [
            ("값 <- 차림[1, 차림[2, 3]].".to_string(), true),
            ("값 <- 차림[1, (차림[2]) 길이].".to_string(), false),
            ("값 <- (첫째:1, 둘째:((차림[2]) 길이)).".to_string(), false),
            (format!("값 <- 차림[{}].", vec!["1"; 129].join(",")), false),
        ] {
            let program = DdnProgram::from_v25_source(&source, "snapshot-container.ddn").unwrap();
            let Stmt::Mutate { value, .. } = &program.root_stmts[0] else { panic!("assignment"); };
            assert_eq!(expression_preserves_local_bindings(value), expected, "{source}");
            // A field lookup itself cannot mutate, but must not hide an
            // effectful/deep base expression from the bounded proof.
            let lookup = Expr::new(value.id, value.span, ExprKind::FieldAccess {
                target: Box::new(value.clone()), field: "첫째".into(), field_span: value.span,
            });
            assert_eq!(expression_preserves_local_bindings(&lookup), expected, "field: {source}");
        }
    }

    #[test]
    fn body_snapshot_proof_keeps_calls_indexed_writes_and_compound_statements() {
        for (source, expected) in [
            ("값 := 차림[1,2].", true),
            ("값 <- (첫째:1,둘째:2).", true),
            ("값 <- 1 / 0.", true), // ordinary Err; never a successful abort
            ("값 <- (차림[1,2]) 길이.", false),
            ("값[0] <- 1.", false),
            ("{ 참 }이면 { 값 <- 1. }.", false),
            ("{ 참 }인 동안 { 멈추기. }.", false),
            ("값 보여주기.", false),
        ] {
            let program=DdnProgram::from_v25_source(source,"body-snapshot.ddn").unwrap();
            assert_eq!(statement_cannot_abort(&program.root_stmts[0]),expected,"{source}");
        }
        let source=format!("값 := 차림[{}].",vec!["1";129].join(","));
        let program=DdnProgram::from_v25_source(&source,"body-snapshot-bound.ddn").unwrap();
        assert!(!statement_cannot_abort(&program.root_stmts[0]));
    }

    #[test]
    fn source_span_cache_preserves_unicode_crlf_and_invalid_offsets() {
        let source = "// 위치 🧪\r\n1 보여주기.\r\n";
        let program = DdnProgram::from_v25_source(source, "위치.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let context = EvalContext::new(&program, &world, &defaults, None,
            InputState::new(0, 0), 0, 0, None);
        // Include non-character-boundary byte offsets: preserve the old
        // projection exactly rather than silently changing its convention.
        for start in 0..=source.len() {
            let end = source.len();
            let (start_line, start_col) = position_to_line_col(source, start);
            let (end_line, end_col) = position_to_line_col(source, end);
            let expected = Some(SourceSpan {
                file: program.program.origin.file_path.clone(),
                start_line, start_col: Some(start_col),
                end_line, end_col: Some(end_col),
            });
            let span = ddonirang_lang::Span { start, end };
            assert_eq!(context.source_span_from_span(&span), expected);
            assert_eq!(context.source_span_from_span(&span), expected);
        }
        assert_eq!(context.source_span_cache.borrow().len(), source.len() + 1);
        for span in [
            ddonirang_lang::Span { start: source.len() + 1, end: 0 },
            ddonirang_lang::Span { start: 0, end: source.len() + 1 },
        ] {
            assert_eq!(context.source_span_from_span(&span), None);
        }
        assert_eq!(context.source_span_cache.borrow().len(), source.len() + 1);
        let empty = DdnProgram::from_v25_source("", "empty.ddn").unwrap();
        let empty_context = EvalContext::new(&empty, &world, &defaults, None,
            InputState::new(0, 0), 0, 0, None);
        assert_eq!(empty_context.source_span_from_span(&ddonirang_lang::Span { start: 0, end: 0 }), None);
        assert!(empty_context.source_span_cache.borrow().is_empty());
    }

    #[test]
    fn wo09_formula_number_reentry_preserves_exact_signed_raw() {
        for raw in [i64::MIN, -1, 0, 1, 1i64 << 25, i64::MAX] {
            let value = Fixed64::from_raw_i64(raw);
            let text = format_formula_expr(&FormulaExpr::Number(
                ddonirang_core::UnitValue { value, dim: ddonirang_core::UnitDim::NONE }), 0);
            assert_eq!(Fixed64::parse_decimal_exact(&text).unwrap().raw_i64(), raw);
            let fields = BTreeMap::from([("값".to_string(), Value::Fixed64(value))]);
            assert_eq!(endpoint_boundary_value_number(&fields).unwrap().formula_text, text);
        }
    }

    #[test]
    fn wo09_exact_value_canon_keeps_adjacent_keys_and_signed_units() {
        let a = Value::Fixed64(Fixed64::from_raw_i64(-1));
        let b = Value::Fixed64(Fixed64::ZERO);
        assert_ne!(value_canon(&a), value_canon(&b));
        assert_eq!(value_canon(&a), "-0.00000000023283064365386962890625");
        assert_eq!(value_canon(&Value::Fixed64(Fixed64::from_raw_i64(-(1i64 << 31)))), "-0.5");
        let unit = Value::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_raw_i64(-1), ddonirang_core::Unit::Second));
        assert_eq!(value_canon(&Value::List(vec![a, unit])),
            "차림[-0.00000000023283064365386962890625, -0.00000000023283064365386962890625@s]");
    }

    #[test]
    fn wo09_resource_bindings_snapshot_replace_remove_restore_matches_deep_map() {
        let mut original = HashMap::from([
            ("output_log".to_string(), Value::List(vec![Value::String("prefix".into())])),
            ("값".to_string(), Value::Fixed64(Fixed64::from_i64(1))),
        ]);
        let mut shared = ResourceBindings::from(original.clone());
        let old_deep = original.clone();
        let old_shared = shared.clone();
        assert!(Arc::ptr_eq(shared.0.get("output_log").unwrap(), old_shared.0.get("output_log").unwrap()));
        let mut old_list = match &mut original.remove("output_log").unwrap() {
            Value::List(items) => std::mem::take(items), _ => panic!("list"),
        };
        let mut new_list = match &mut shared.remove("output_log").unwrap() {
            Value::List(items) => std::mem::take(items), _ => panic!("list"),
        };
        old_list.push(Value::String("next".into()));
        new_list.push(Value::String("next".into()));
        original.insert("output_log".into(), Value::List(old_list));
        shared.insert("output_log".into(), Value::List(new_list));
        original.insert("값".into(), Value::Fixed64(Fixed64::from_i64(2)));
        shared.insert("값".into(), Value::Fixed64(Fixed64::from_i64(2)));
        let canonical = |values: &HashMap<String, Value>| values.iter().map(|(key, value)|
            (key.clone(), resource_value_from_value(value).unwrap())).collect::<BTreeMap<_, _>>();
        assert_eq!(canonical(&original), canonical(&shared.clone().into_owned()));
        assert_eq!(canonical(&old_deep), canonical(&old_shared.clone().into_owned()));
        original = old_deep;
        shared = old_shared;
        assert_eq!(canonical(&original), canonical(&shared.into_owned()));
    }

    // Same-control-flow replay only: both contexts use shared bindings/PatchLog.
    // Separate deep-storage tests and the official baseline cover old bytes.
    fn wo09_output_snapshot_replay_body(
        ctx: &mut EvalContext<'_>,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<FlowControl, EvalError> {
        if ctx.aborted { return Ok(FlowControl::Continue); }
        for stmt in &body.stmts {
            if ctx.aborted { return Ok(FlowControl::Continue); }
            let _registry_key = stmt_registry_key(stmt);
            ctx.work_cost.borrow_mut().charge_dispatch()?;
            let snapshot = ctx.capture_frame_snapshot(locals);
            match ctx.eval_stmt(locals, stmt)? {
                FlowControl::Continue => {}
                other => return Ok(other),
            }
            if ctx.aborted {
                ctx.restore_frame_snapshot_preserving_abort_contract_diag(locals, &snapshot);
                return Ok(FlowControl::Continue);
            }
        }
        Ok(FlowControl::Continue)
    }

    fn wo09_output_snapshot_compare(
        program: &DdnProgram,
        body: &Body,
        initial: HashMap<String, Value>,
        counters: crate::work_cost::WorkCostCounters,
    ) -> (HashMap<String, Value>, Vec<PatchOp>, bool, Option<String>) {
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut fast = EvalContext::new(program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let mut legacy = EvalContext::new(program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        for ctx in [&mut fast, &mut legacy] {
            ctx.resources = initial.clone().into();
            *ctx.work_cost.borrow_mut() = WorkCostMeter::from_receipt_parts(
                counters.clone(), StandardResourceLimitsV1::CANDIDATE, false,
            );
        }
        let mut fast_locals = HashMap::new();
        let mut legacy_locals = HashMap::new();
        // This comparator feeds the program root statement list, not a lexical child block.
        let fast_error = fast.eval_body_unscoped(&mut fast_locals, body).err().map(|e| e.message());
        let legacy_error = wo09_output_snapshot_replay_body(&mut legacy, &mut legacy_locals, body)
            .err().map(|e| e.message());
        assert_eq!(fast_error, legacy_error);
        assert_eq!(fast.patch_ops, legacy.patch_ops);
        assert_eq!(fast.aborted, legacy.aborted);
        assert_eq!(fast.guard_rejected, legacy.guard_rejected);
        assert_eq!(fast.input_reads.borrow().clone(), legacy.input_reads.borrow().clone());
        assert_eq!(fast.work_cost.borrow().receipt(), legacy.work_cost.borrow().receipt());
        let convert = |values: &HashMap<String, Value>| values.iter().map(|(key, value)|
            (key.clone(), resource_value_from_value(value).expect("test scalar/list resource"))
        ).collect::<BTreeMap<_, _>>();
        assert_eq!(convert(&fast.resources.clone().into_owned()), convert(&legacy.resources.clone().into_owned()));
        assert_eq!(convert(&fast_locals), convert(&legacy_locals));
        let mut fast_world = DetNuri::new();
        let mut legacy_world = DetNuri::new();
        fast_world.apply_patch(&Patch { ops: fast.patch_ops.clone().into_owned(), ..Patch::default() }, 0,
            &mut ddonirang_core::signals::VecSignalSink::default());
        legacy_world.apply_patch(&Patch { ops: legacy.patch_ops.clone().into_owned(), ..Patch::default() }, 0,
            &mut ddonirang_core::signals::VecSignalSink::default());
        assert_eq!(fast_world.state_hash(), legacy_world.state_hash());
        (fast.resources.into_owned(), fast.patch_ops.into_owned(), fast.guard_rejected, fast_error)
    }

    fn wo09_output_snapshot_body(program: &DdnProgram) -> Body {
        Body { id: 0, span: ddonirang_lang::Span { start: 0, end: 0 },
            stmts: program.root_stmts.clone() }
    }

    #[test]
    fn wo09_output_snapshot_plain_literals_and_vars_match_replay() {
        let mut program = DdnProgram::from_v25_source(
            r#"값 := 3. 값 보여주기. "표" 보여주기. 1 보여주기. 0.5 보여주기. 참 보여주기. 없음 보여주기."#,
            "wo09-output-snapshot.ddn",
        ).unwrap();
        let Stmt::Show { id, span, mood, expr } = program.root_stmts[2].clone() else {
            panic!("string output");
        };
        program.root_stmts[2] = Stmt::Inspect { id, span, mood, expr };
        let (_, patch, _, error) = wo09_output_snapshot_compare(&program,
            &wo09_output_snapshot_body(&program), HashMap::new(), Default::default());
        assert!(error.is_none());
        assert_eq!(patch.iter().filter(|p| matches!(p,
            PatchOp::SetResourceValue { tag, .. } if tag == "output_log")).count(), 6);
    }

    #[test]
    fn a3_p2_local_derived_unit_rebinding_rejects_another_dimension() {
        // A3-P2: actual V25 source through the shared product parser/runtime.
        let source = "검산:움직씨 := { 면적 <- 2@m * 3@m. 면적 <- 1@s. }.\n() 검산하기.";
        let program = DdnProgram::from_v25_source(source, "unit-local.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .err().expect("dimension-changing rebinding must fail");
        assert!(error.to_string().contains("씨 불일치") || error.to_string().contains("TYPE_MISMATCH"), "{error}");
    }

    #[test]
    fn a3_p2_local_derived_unit_rebinding_keeps_same_dimension() {
        let source = "검산:움직씨 := { 면적 <- 2@m * 3@m. 면적 <- 4@m * 5@m. 면적 보여주기. }.\n() 검산하기.";
        let program = DdnProgram::from_v25_source(source, "unit-local-same.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        assert!(output.display_outputs.iter().any(|row| row.text == "20@m^2"), "{:?}", output.display_outputs);
    }

    #[test]
    fn a3_p1_output_display_snapshot_rollback_preserves_world_text() {
        let program = DdnProgram::from_v25_source("1 보여주기.", "display-snapshot.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let mut locals = HashMap::new();
        ctx.append_output_log(Value::String("19.019999999".into())).unwrap();
        let saved = ctx.capture_frame_snapshot(&locals);
        ctx.append_output_log(Value::Fixed64(Fixed64::parse_decimal_exact("19.02").unwrap())).unwrap();
        assert_eq!(ctx.display_outputs[1].text, "19.02");
        ctx.restore_frame_snapshot(&mut locals, &saved);
        assert_eq!(ctx.display_outputs, saved.display_outputs);
        assert_eq!(ctx.display_outputs.len(), 1);
        assert_eq!(ctx.display_outputs[0].text, "19.019999999");
        assert_eq!(ctx.patch_ops, saved.patch_ops);
        assert_eq!(ctx.resources.clone().into_owned(), saved.resources.clone().into_owned());
    }

    #[test]
    fn wo09_output_snapshot_complex_call_abort_keeps_rollback() {
        let mut program = DdnProgram::from_v25_source(
            r#"고장:움직씨 := {
                99 보여주기.
                { 거짓 }인것 바탕으로(물림) 아니면 { 없음. }
            }
            7 보여주기. 1 보여주기. 8 보여주기."#,
            "wo09-output-call-abort.ddn",
        ).unwrap();
        let callable = program.functions.values().find(|seed| seed.body.as_ref()
            .is_some_and(|body| body.stmts.iter().any(|stmt| matches!(stmt, Stmt::Contract { .. }))))
            .expect("parsed aborting callable must be registered").canonical_name.clone();
        let Stmt::Show { expr, .. } = &mut program.root_stmts[1] else { panic!("call output"); };
        // Existing normalized Call AST; no test-only source lowering/frontdoor.
        expr.kind = ExprKind::Call { func: callable, args: Vec::new() };
        let (resources, patch, _, error) = wo09_output_snapshot_compare(&program,
            &wo09_output_snapshot_body(&program), HashMap::new(), Default::default());
        assert!(error.is_none(), "complex call error: {error:?}");
        let Value::List(log) = resources.get("output_log").unwrap() else { panic!("prior output"); };
        assert_eq!(log.len(), 1, "aborted callable must not publish its output or following output");
        assert!(matches!(patch.first(), Some(PatchOp::SetResourceValue { tag, .. }) if tag == "output_log"));
        assert!(matches!(patch.last(), Some(PatchOp::EmitSignal { signal: Signal::Diag { event }, .. })
            if event.rule_id == "L0-CONTRACT-01" && event.mode.as_deref() == Some("물림")));
    }

    #[test]
    fn wo09_output_snapshot_guard_failure_keeps_prefix_clear() {
        let mut program = DdnProgram::from_v25_source("7 보여주기. 8 보여주기.", "wo09-output-guard.ddn").unwrap();
        let mut body = wo09_output_snapshot_body(&program);
        let span = ddonirang_lang::Span { start: 0, end: 0 };
        body.stmts[1] = Stmt::Guard {
            id: 900, span, mood: ddonirang_lang::Mood::Declarative,
            condition: Expr::new(901, span, ExprKind::Literal(Literal::Bool(true))),
            body: Body { id: 902, span, stmts: Vec::new() },
        };
        program.root_stmts = body.stmts.clone();
        let (_, patch, rejected, error) = wo09_output_snapshot_compare(&program, &body,
            HashMap::new(), Default::default());
        assert!(error.is_none());
        assert!(rejected);
        assert!(!patch.iter().any(|p| matches!(p, PatchOp::SetResourceValue { tag, .. } if tag == "output_log")));
        assert!(patch.iter().any(|p| matches!(p, PatchOp::GuardViolation { .. })));
    }

    #[test]
    fn wo09_output_snapshot_meter_overflow_matches_replay_error() {
        let program = DdnProgram::from_v25_source("1 보여주기.", "wo09-output-meter.ddn").unwrap();
        let counters = crate::work_cost::WorkCostCounters {
            work_cost: u64::MAX - 1, ..Default::default()
        };
        let (_, patch, _, error) = wo09_output_snapshot_compare(&program,
            &wo09_output_snapshot_body(&program), HashMap::new(), counters);
        assert!(error.unwrap().contains("E_WORK_COST_OVERFLOW_CANDIDATE"));
        assert!(patch.is_empty());
    }

    #[test]
    fn wo09_output_snapshot_var_keeps_input_read_tracking() {
        let program = DdnProgram::from_v25_source("입력키 보여주기.", "wo09-output-input.ddn").unwrap();
        let initial = HashMap::from([("입력키".to_string(), Value::String("ArrowLeft".to_string()))]);
        wo09_output_snapshot_compare(&program, &wo09_output_snapshot_body(&program), initial,
            Default::default());
        let mut runner = DdnRunner::new(program, "매마디");
        let mut input = empty_input();
        input.last_key_name = "ArrowLeft".to_string();
        let output = runner.run_update(&NuriWorld::new(), &input, &HashMap::new()).unwrap();
        assert!(output.input_reads.contains("last_key_name"));
    }


    fn wo09_patch_bytes(ops: Vec<PatchOp>) -> Vec<u8> {
        ddonirang_core::canonical_patch_ops_text(&Patch { ops, ..Patch::default() }).into_bytes()
    }

    #[test]
    fn wo09_patch_log_snapshot_append_restore_matches_deep_vec_bytes() {
        let mut old = Vec::new();
        let mut log = PatchLog::default();
        for value in 0..4 {
            let op = PatchOp::SetResourceValue {
                tag: "output_log".to_string(),
                value: ResourceValue::List(vec![ResourceValue::String("가".repeat(1000)); value + 1]),
            };
            old.push(op.clone());
            log.push(op);
        }
        let deep_snapshot = old.clone();
        let snapshot = log.clone();
        assert!(Arc::ptr_eq(&log.0[0], &snapshot.0[0]), "snapshot must share immutable payload");
        let op = PatchOp::SetResourceFixed64 { tag: "x".to_string(), value: Fixed64::ONE };
        old.push(op.clone()); log.push(op);
        assert_eq!(wo09_patch_bytes(log.clone().into_owned()), wo09_patch_bytes(old.clone()));
        old.clear(); log.clear();
        assert!(log.is_empty());
        old = deep_snapshot; log = snapshot;
        assert_eq!(wo09_patch_bytes(log.into_owned()), wo09_patch_bytes(old));
    }

    #[test]
    fn wo09_patch_log_guard_clear_then_snapshot_restore_retains_prefix() {
        let program = DdnProgram::from_v25_source("7 보여주기.", "wo09-patch-guard.ddn").unwrap();
        let world = NuriWorld::new(); let defaults = HashMap::new();
        let mut ctx = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let mut locals = HashMap::new();
        ctx.eval_body(&mut locals, &wo09_output_snapshot_body(&program)).unwrap();
        let deep_prefix = ctx.patch_ops.clone().into_owned();
        let snapshot = ctx.capture_frame_snapshot(&locals);
        let span = ddonirang_lang::Span { start: 0, end: 0 };
        let guard = Stmt::Guard { id: 900, span, mood: ddonirang_lang::Mood::Declarative,
            condition: Expr::new(901, span, ExprKind::Literal(Literal::Bool(true))),
            body: Body { id: 902, span, stmts: Vec::new() } };
        ctx.eval_stmt(&mut locals, &guard).unwrap();
        assert!(ctx.guard_rejected);
        assert!(ctx.patch_ops.iter().all(|op| !matches!(op, PatchOp::SetResourceValue { tag, .. } if tag == "output_log")));
        ctx.restore_frame_snapshot(&mut locals, &snapshot);
        assert!(!ctx.guard_rejected);
        assert_eq!(wo09_patch_bytes(ctx.patch_ops.into_owned()), wo09_patch_bytes(deep_prefix));
    }

    #[test]
    fn wo09_patch_log_child_success_and_failure_keep_parent_prefix_isolated() {
        for (body, fails) in [
            ("입력키 보여주기. 값 <- 3.", false),
            ("입력키 보여주기. 값 <- 3. { 거짓 }인것 바탕으로(물림) 아니면 { 없음. }", true),
        ] {
            let source = format!("조치:움직씨 := {{ {body} }} 7 보여주기.");
            let program = DdnProgram::from_v25_source(&source, "wo09-patch-child.ddn").unwrap();
            let seed = program.functions.values().find(|seed| seed.body.as_ref()
                .is_some_and(|body| body.stmts.iter().any(|stmt| matches!(stmt, Stmt::Mutate { .. }))))
                .expect("parsed action");
            let world = NuriWorld::new(); let defaults = HashMap::new();
            let mut ctx = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
            ctx.resources.insert("입력키".to_string(), Value::String("ArrowLeft".to_string()));
            let mut locals = HashMap::new();
            ctx.eval_body(&mut locals, &wo09_output_snapshot_body(&program)).unwrap();
            let prefix = ctx.patch_ops.clone();
            let old_bytes = wo09_patch_bytes(prefix.clone().into_owned());
            let before_cost = ctx.work_cost.borrow().receipt();
            let result = ctx.eval_state_machine_transition_action(&seed.canonical_name, &HashMap::new());
            assert_eq!(result.is_err(), fails, "child result: {result:?}");
            assert_eq!(wo09_patch_bytes(prefix.into_owned()), old_bytes, "child cannot mutate captured prefix");
            assert_ne!(ctx.work_cost.borrow().receipt(), before_cost, "child work remains accounted");
            assert!(ctx.input_reads.borrow().contains("last_key_name"), "child input reads remain accounted");
            if fails {
                assert!(result.err().unwrap().message().contains("E_STATE_TRANSITION_ACTION_ABORTED"));
                assert_eq!(wo09_patch_bytes(ctx.patch_ops.into_owned()), old_bytes);
                assert!(!ctx.resources.contains_key("값"));
            } else {
                assert!(ctx.resources.contains_key("값"));
                assert!(ctx.patch_ops.len() > 1);
                assert!(matches!(ctx.patch_ops.iter().next(), Some(PatchOp::SetResourceValue { tag, .. }) if tag == "output_log"));
            }
        }
    }

    static AGE_TARGET_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct AgeTargetTestGuard {
        _lock: MutexGuard<'static, ()>,
        previous: AgeTarget,
    }

    impl AgeTargetTestGuard {
        fn new(age: AgeTarget) -> Self {
            let lock = AGE_TARGET_TEST_LOCK
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            let previous = default_age_target();
            set_default_age_target(age);
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for AgeTargetTestGuard {
        fn drop(&mut self) {
            set_default_age_target(self.previous);
        }
    }

    struct FactorizationTestOverrideGuard {
        previous_iters: usize,
        previous_trial_limit: u64,
    }

    impl FactorizationTestOverrideGuard {
        fn new(iter_limit: usize, trial_limit: u64) -> Self {
            Self {
                previous_iters: swap_factor_pollard_max_iters_for_test(iter_limit),
                previous_trial_limit: swap_factor_trial_fallback_limit_for_test(trial_limit),
            }
        }
    }

    impl Drop for FactorizationTestOverrideGuard {
        fn drop(&mut self) {
            swap_factor_pollard_max_iters_for_test(self.previous_iters);
            swap_factor_trial_fallback_limit_for_test(self.previous_trial_limit);
        }
    }

    fn empty_input() -> InputSnapshot {
        InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            frame_source: ddonirang_core::InputSource::Person,
            rng_seed: 0,
        }
    }

    #[test]
    fn i1_input_reads_follow_execution_not_source_presence() {
        for (source, read) in [
            ("1 보여주기.", false),
            ("입력키 보여주기.", true),
            ("검사값 := 거짓 그리고 (입력키 == \"ArrowLeft\"). 검사값 보여주기.", false),
            ("검사값 := 참 그리고 (입력키 == \"ArrowLeft\"). 검사값 보여주기.", true),
        ] {
            let program = DdnProgram::from_v25_source(source, "i1-input.ddn").unwrap();
            let mut runner = DdnRunner::new(program, "매마디");
            let mut input = empty_input();
            input.last_key_name = "ArrowLeft".into();
            let output = runner.run_update(&NuriWorld::new(), &input, &HashMap::new()).unwrap();
            assert_eq!(output.input_reads.contains("last_key_name"), read, "{source}");
            assert_eq!(crate::input_receipt::validate_tape_v1_reads(&output.input_reads).is_err(), read);
        }
    }

    #[test]
    fn i1_builtin_resource_access_tracks_input_read() {
        let program = DdnProgram::from_v25_source("1 보여주기.", "i1.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut context = EvalContext::new(&program, &world, &defaults, None,
            InputState::new(0, 0), 0, 0, None);
        context.resources.insert("입력키".into(), Value::String("ArrowLeft".into()));
        // 내장 보개 값 해소의 내부 자원 읽기. 새 테스트 전용 접근 경로를 만들지 않는다.
        let value = context.parse_bogae_value_token(&mut HashMap::new(), "입력키");
        assert!(matches!(value, Some(Value::String(ref value)) if value == "ArrowLeft"));
        assert!(crate::input_receipt::validate_tape_v1_reads(&context.input_reads.borrow()).is_err());
    }

    #[test]
    fn d9_explicit_entry_sees_initial_values_and_does_not_repeat_on_finish() {
        let program = DdnProgram::from_v25_source(
            "중력:수 := 9.8. 실행:움직씨 := { 중력 보여주기. }", "entry.ddn",
        ).unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        assert!(runner.configure_project_entry("없는시작").is_err());
        runner.configure_project_entry("실행").unwrap();
        let world = DetNuri::new();
        let output = runner.run_update(world.world(), &empty_input(), &HashMap::new()).unwrap();
        assert!(output.resources.contains_key("중력"));
        let finish = runner.run_finish(world.world(), &empty_input(), &HashMap::new()).unwrap();
        assert!(finish.patch.ops.is_empty());
        assert!(!finish.resources.contains_key("중력"));
    }

    #[test]
    fn a3_p12_project_entry_keeps_top_level_setup_across_ticks() {
        let program = DdnProgram::from_v25_source(
            "채비 { 진행:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 진행 <- 진행 + 1. }. }.",
            "project-setup.ddn",
        ).unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        runner.configure_project_entry("실행").unwrap();
        let mut world = DetNuri::new();
        for tick_id in 0..3 {
            let mut input = empty_input(); input.tick_id = tick_id;
            let output = runner.run_update(world.world(), &input, &HashMap::new()).unwrap();
            assert_eq!(extract_fixed(&output.resources, "진행"), Fixed64::from_i64(tick_id as i64 + 1));
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
        }
    }

    #[test]
    fn a3_p12_project_setup_does_not_invoke_other_tick_or_callable_locals() {
        let program = DdnProgram::from_v25_source(
            "채비 { 진행:수 := 0. }. 매마디:움직씨 := { 채비 { 지역값:수 := 9. }. 진행 <- 999. }. 채비 { 뒤값:수 := 7. }. 실행:움직씨 := { 진행 <- 진행 + 뒤값. }.",
            "project-other-tick.ddn",
        ).unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        runner.configure_project_entry("실행").unwrap();
        let output = runner.run_update(DetNuri::new().world(), &empty_input(), &HashMap::new()).unwrap();
        assert_eq!(extract_fixed(&output.resources, "진행"), Fixed64::from_i64(7));
        assert!(!output.resources.contains_key("지역값"));
    }

    #[test]
    fn wo03_v25_arithmetic_failure_is_not_a_successful_signal() {
        for source in [
            "(1 / 0) 보여주기.",
            "값 <- (1 / 0). 값 보여주기.",
            "값 := (1 / 0). 값 보여주기.",
            "(매마디)마다 { (1 / 0) 보여주기. }.",
            "(매마디)마다 { 값 <- (1 / 0). 값 보여주기. }.",
            "고장:움직씨 := { (1 / 0) 보여주기. } () 고장.",
        ] {
            let program = DdnProgram::from_v25_source(source, "wo03.ddn").expect(source);
            let mut runner = DdnRunner::new(program, "매마디");
            let result = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new());
            assert!(result.is_err(), "must fail instead of publishing a frame: {source}");
            assert!(result.err().unwrap().contains("0으로 나눌 수 없습니다"), "{source}");
        }
    }

    #[test]
    fn wo02_short_circuit_skips_only_the_unneeded_operand() {
        for (expression, expected) in [
            ("거짓 그리고 ((1 / 0) == 0)", false),
            ("참 또는 ((1 / 0) == 0)", true),
        ] {
            let source = format!("검사값 <- {expression}.");
            let program = DdnProgram::from_v25_source(&source, "wo02.ddn").expect(&source);
            let mut runner = DdnRunner::new(program, "매마디");
            let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect(&source);
            assert!(matches!(output.resources.get("검사값"), Some(RuntimeValue::Bool(v)) if *v == expected), "{source}: {:?}", output.resources);
        }
        for expression in ["참 그리고 ((1 / 0) == 0)", "거짓 또는 ((1 / 0) == 0)"] {
            let source = format!("결과 := {expression}.");
            let program = DdnProgram::from_v25_source(&source, "wo02.ddn").expect(&source);
            let mut runner = DdnRunner::new(program, "매마디");
            assert!(runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).is_err(), "{source}");
        }
    }

    #[test]
    fn wo02_short_circuit_does_not_publish_rhs_effects() {
        let source = "부작용:움직씨 := { 99 보여주기. } 검사값 <- (거짓 그리고 (() 부작용)).";
        let program = DdnProgram::from_v25_source(source, "wo02.ddn").expect("source");
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("skip effect");
        assert!(matches!(output.resources.get("검사값"), Some(RuntimeValue::Bool(false))));
        assert!(!output.resources.contains_key("output_log"));
    }

    #[test]
    fn wo02_foreach_type_rejection_restores_outer_binding() {
        let rejected="항목 <- \"바깥\". (항목) (1, 2) 차림에 대해 { 항목 <- \"가\". }.";
        assert!(DdnProgram::from_v25_source(rejected,"wo02-static.ddn").is_err(),"authored type conflict must still reject");
        let source = "항목 <- \"바깥\". 자료 <- 차림[\"안\",\"둘\"]. (항목) 자료에 대해 { 항목 <- \"가\". }.";
        let program = DdnProgram::from_v25_source(source, "wo02.ddn").expect("source");
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let mut locals = HashMap::new();
        ctx.eval_stmt(&mut locals, &program.root_stmts[0]).expect("outer binding");
        // Deliberately corrupt only this private evaluator fixture after
        // static admission. Runtime defenses must restore the outer scope,
        // even for an impossible source-level iteration value.
        locals.insert("자료".into(),Value::List(vec![Value::Fixed64(Fixed64::from_i64(1))]));
        let before = ctx.binding_type_scopes.clone();
        let error = ctx.eval_stmt(&mut locals, &program.root_stmts[2]).err().expect("different iteration type");
        assert!(error.to_string().contains("TYPE_MISMATCH"), "{error}");
        assert_eq!(ctx.binding_type_scopes, before);
    }

    #[test]
    fn wo02_internal_call_ast_uses_short_circuit_without_new_surface() {
        for (op, expected) in [("그리고", false), ("또는", true)] {
            let literal = if expected { "참" } else { "거짓" };
            let source = format!("검사값 <- {literal} {op} ((1 / 0) == 0).");
            let mut program = DdnProgram::from_v25_source(&source, "wo02.ddn").expect("infix source");
            let mut infix_runner = DdnRunner::new(program.clone(), "매마디");
            infix_runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("infix run");
            let infix_cost = infix_runner.work_cost_receipt().counters.base_dispatches;
            let Stmt::Mutate { value, .. } = &mut program.root_stmts[0] else { panic!("binding") };
            let ExprKind::Infix { left, right, .. } = value.kind.clone() else { panic!("infix") };
            value.kind = ExprKind::Call {
                func: op.to_string(),
                args: [*left, *right].into_iter().map(|expr| ddonirang_lang::ArgBinding {
                    id: expr.id, span: expr.span, expr, josa: None, resolved_pin: None,
                    binding_reason: ddonirang_lang::BindingReason::Positional,
                }).collect(),
            };
            let mut runner = DdnRunner::new(program, "매마디");
            let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("internal call short circuit");
            assert!(matches!(output.resources.get("검사값"), Some(RuntimeValue::Bool(v)) if *v == expected));
            // Existing call dispatch has one charge; an infix node does not.
            // No RHS division dispatch or RHS error is observed in either path.
            assert_eq!(runner.work_cost_receipt().counters.base_dispatches, infix_cost + 1);
        }
    }

    #[test]
    fn wo02_inferred_collection_category_does_not_claim_element_type_inference() {
        let source = "값 := (1, 2) 차림. 값 <- (\"가\", \"나\") 차림. 값 보여주기.";
        let program = DdnProgram::from_v25_source(source, "wo02.ddn").expect("source");
        let mut runner = DdnRunner::new(program, "매마디");
        runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("existing collection-level category");
    }

    #[test]
    fn wo02_inferred_local_and_first_binding_types_are_fixed() {
        for body in ["값 := 1. 값 <- \"가\".", "값 <- 1. 값 <- \"가\"."] {
            let source = format!("검산:움직씨 := {{ {body} }}");
            let program = DdnProgram::from_v25_source(&source, "wo02.ddn").expect(&source);
            let mut runner = DdnRunner::new(program, "검산");
            let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("reject different type");
            assert!(error.to_string().contains("TYPE_MISMATCH"), "{error}");
        }
    }

    #[test]
    fn wo03_list_lookup_is_zero_based_and_preserves_absence() {
        for (index, present) in [(0, true), (1, true), (2, false), (-1, false)] {
            let source = format!("목록 <- (10, 20) 차림. p <- ((목록, {index}) 찾기) 있는가.");
            let program = DdnProgram::from_v25_source(&source, "wo03.ddn").expect("parse");
            let output = DdnRunner::new(program, "매마디")
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("lookup");
            assert!(matches!(output.resources.get("p"), Some(RuntimeValue::Bool(value)) if *value == present));
        }
        for source in [
            "목록 <- (10, 20) 차림. (목록, 0.5) 찾기 보여주기.",
            "목록 <- (없음, 20) 차림. (목록, 0) 찾기 보여주기.",
        ] {
            let program = DdnProgram::from_v25_source(source, "wo03.ddn").expect("parse");
            assert!(DdnRunner::new(program, "매마디")
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).is_err());
        }
    }

    #[test]
    fn wo03_text_lookup_never_substitutes_codepoints_or_legacy_substring() {
        for expression in ["(\"가\", 0) 찾기", "(\"é\", 0) 찾기", "(\"가나\", \"나\") 찾기"] {
            let source = format!("결과 <- {expression}. 결과 보여주기.");
            let program = DdnProgram::from_v25_source(&source, "wo03.ddn").expect("parse");
            let error = DdnRunner::new(program, "매마디")
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("no silent substitution");
            assert!(error.contains("E_RUNTIME_TEXT_LOOKUP_UNSUPPORTED"), "{error}");
        }
    }

    #[test]
    fn wo02_stored_none_lookup_fails_closed_but_missing_key_remains_absent() {
        for function in ["찾기", "찾기?", "짝맞춤.값"] {
            let source = format!("m <- (\"a\", 없음) 짝맞춤. p <- ((m, \"a\") {function}) 있는가.");
            let program = DdnProgram::from_v25_source(&source, "wo02.ddn").expect(&source);
            let mut runner = DdnRunner::new(program, "매마디");
            let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("stored None must not become absent");
            assert!(error.to_string().contains("E_RUNTIME_NESTED_OPTIONAL_UNSUPPORTED"), "{error}");
        }
        let source = "m <- (\"a\", 없음) 짝맞춤. p <- ((m, \"zz\") 찾기) 있는가.";
        let program = DdnProgram::from_v25_source(source, "wo02.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).expect("missing key");
        assert!(matches!(output.resources.get("p"), Some(RuntimeValue::Bool(false))));
    }

    #[test]
    fn v25_definition_then_rebinding_executes_as_distinct_local_operations() {
        let script = crate::self_contained_test_sources::V25_DEFINITION_AND_BINDING;
        let program = DdnProgram::from_v25_source(script, "pc01_runtime.ddn")
            .expect("V25 product runtime parser");
        let mut runner = DdnRunner::new(program, "검산");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("V25 definition/binding execution");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("output_log must be list");
        };
        let RuntimeValue::Map(map) = entries.first().expect("output log entry") else {
            panic!("output log entry must be map");
        };
        let text = map
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|entry| match &entry.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("7"));
    }

    #[test]
    fn v25_root_binding_rebinding_executes_without_a_synthetic_callable() {
        let source = "값 <- 0.\n값 <- 값 + 1.\n값 보여주기.\n";
        let program = DdnProgram::from_v25_source(source, "root_binding.ddn")
            .expect("V25 root program parses");
        assert!(program.has_root_program());
        assert!(program.functions.is_empty());

        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("root binding program executes");
        assert_eq!(extract_fixed(&output.resources, "값"), Fixed64::from_i64(1));

        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("root show must publish an output log entry");
        };
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn alpha3_rd136_admits_time_tokens_without_text_or_namespace_heuristics() {
        for source in [
            "글 := \"표준시간.고정마디\".",
            "결과 := () 다른.표준시간.고정마디.",
        ] {
            DdnProgram::from_v25_source(source,"text-time.ddn").expect("no token-level time guard");
        }
        for source in [
            "시간 := (번호=2) 표준시간.고정마디.",
            "시간 := (차이=-1) 표준시간.상대마디.",
        ] {
            DdnProgram::from_v25_source(source, "time.ddn").expect("RD-136 constructors now admitted by Alpha3");
        }
    }

    #[test]
    fn alpha2_primary_relation_accepts_only_a_typed_view() {
        let source = "보기:보기값 := () 표준보기.결과표. 기본:으뜸보개관계값 := (보기=보기) 표준보개.으뜸. 기본 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_primary.ddn")
            .expect("official V25 named-pin source");
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("typed primary constructor");
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else {
            panic!("show must publish a text row");
        };
        let RuntimeValue::Map(row) = &rows[0] else { panic!("output row"); };
        assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
            (RuntimeValue::String(key), RuntimeValue::String(text))
                if key == "text" && text == "<으뜸보개관계값>")));
        assert!(output.patch.ops.iter().all(|op| !matches!(op,
            PatchOp::SetResourceJson { tag, .. }
            | PatchOp::SetResourceFixed64 { tag, .. }
            | PatchOp::SetResourceValue { tag, .. } if tag == "기본")));

        let program = DdnProgram::from_v25_source(
            "기본 := (보기=\"결과표\") 표준보개.으뜸.", "alpha2_fake_primary.ddn")
            .expect("source parses without a test-only rewrite");
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .err().expect("plain string rejected");
        assert!(error.to_string().contains("보기값"), "{error}");
    }

    #[test]
    fn alpha2_composition_accepts_typed_flat_relations_without_world_publication() {
        let source = "보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. 짜임:보개짜임값 := (관계들=차림[기본]) 표준보개.짜임. 짜임 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_composition.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else { panic!("output row"); };
        let RuntimeValue::Map(row) = &rows[0] else { panic!("output row map"); };
        assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
            (RuntimeValue::String(key), RuntimeValue::String(text))
                if key == "text" && text == "<보개짜임값>")));
        assert!(output.patch.ops.iter().all(|op| !matches!(op,
            PatchOp::SetResourceJson { tag, .. }
            | PatchOp::SetResourceFixed64 { tag, .. }
            | PatchOp::SetResourceValue { tag, .. } if tag == "짜임")));
    }

    #[test]
    fn alpha2_collection_search_compares_nested_coordinates() {
        let source = "칸들 <- 차림[차림[0, 1], 차림[2, 3]]. ((칸들, 차림[2, 3]) 들어있나) 보여주기. ((칸들, 차림[2, 3]) 찾아보기) 보여주기. ((칸들, 차림[3, 2]) 들어있나) 보여주기. ((칸들, 차림[3, 2]) 찾아보기) 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_coordinate_search.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else { panic!("output rows"); };
        assert_eq!(rows.len(), 4);
        for (row, expected) in rows.iter().zip(["참", "1", "거짓", "-1"]) {
            let RuntimeValue::Map(row) = row else { panic!("output row"); };
            assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
                (RuntimeValue::String(key), RuntimeValue::String(text)) if key == "text" && text == expected)));
        }
    }

    #[test]
    fn alpha2_collection_search_rejects_nested_resource_equality() {
        let program = DdnProgram::from_v25_source("", "alpha2_search.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let context = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let candidate = Value::List(vec![Value::ResourceHandle(ResourceHandle::from_raw(1))]);
        let error = context.eval_collection_search_equality(&candidate, &candidate).unwrap_err().to_string();
        assert!(error.contains("E_VALUE_EQUALITY_UNSUPPORTED"), "{error}");
        assert!(error.contains("왼쪽$[0]"), "{error}");
    }

    #[test]
    fn alpha2_exact_equality_compares_nested_ordered_values() {
        let program = DdnProgram::from_v25_source("", "alpha2_equality.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let runner = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let value = Value::List(vec![Value::List(vec![Value::Bool(true), Value::Bool(false)])]);
        let reversed = Value::List(vec![Value::List(vec![Value::Bool(false), Value::Bool(true)])]);
        assert!(runner.eval_exact_value_equality(&value, &value).unwrap());
        assert!(!runner.eval_exact_value_equality(&value, &reversed).unwrap());
        assert!(!runner.eval_exact_value_equality(&value, &Value::List(vec![])).unwrap());
        for wrap in [0, 1, 2] {
            let fields = BTreeMap::from([("가".to_string(), value.clone())]);
            let wrapped = match wrap {
                0 => Value::Pack(fields),
                1 => Value::Set(fields),
                _ => Value::Map(BTreeMap::from([("가".to_string(), MapEntry {
                    key: Value::String("가".to_string()), value: value.clone(),
                })])),
            };
            assert!(runner.eval_exact_value_equality(&wrapped, &wrapped).unwrap());
        }
    }

    #[test]
    fn alpha2_exact_equality_rejects_unread_later_incomparable_values() {
        let program = DdnProgram::from_v25_source("", "alpha2_equality.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let runner = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let bad = Value::List(vec![Value::Bool(true), Value::List(vec![
            Value::ResourceHandle(ResourceHandle::from_raw(1)),
        ])]);
        let other = Value::List(vec![Value::Bool(false)]);
        for (left, right, operand) in [(&bad, &other, "왼쪽"), (&other, &bad, "오른쪽")] {
            let error = runner.eval_exact_value_equality(left, right).unwrap_err().to_string();
            assert!(error.contains("E_VALUE_EQUALITY_UNSUPPORTED"), "{error}");
            assert!(error.contains(&format!("{operand}$[1][0]")), "{error}");
        }
    }

    #[test]
    fn alpha2_exact_equality_preserves_nested_diagnostic_paths() {
        let program = DdnProgram::from_v25_source("", "alpha2_equality.ddn").unwrap();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let runner = EvalContext::new(&program, &world, &defaults, None, InputState::new(0, 0), 0, 0, None);
        let bad = Value::ResourceHandle(ResourceHandle::from_raw(1));
        let field = "칸\"이름".to_string();
        let packed = Value::Pack(BTreeMap::from([(field.clone(), bad.clone())]));
        let cases = [
            (Value::List(vec![packed.clone()]), format!("[0][{}]", serde_json::to_string(&field).unwrap())),
            (Value::Set(BTreeMap::from([("가".into(), packed)])), format!("[모음:0][{}]", serde_json::to_string(&field).unwrap())),
            (Value::Map(BTreeMap::from([("가".into(), MapEntry { key: bad.clone(), value: Value::Bool(true) })])), "[짝:0].열쇠".into()),
            (Value::Map(BTreeMap::from([("가".into(), MapEntry { key: Value::String("가".into()), value: bad })])), "[짝:0].값".into()),
        ];
        for (value, path) in cases {
            let error = runner.validate_exact_comparison_value(&value, "오른쪽").unwrap_err().to_string();
            assert_eq!(error, format!("E_VALUE_EQUALITY_UNSUPPORTED: 오른쪽${path}의 값은 기본 동등성 비교를 지원하지 않습니다"));
        }
    }

    #[test]
    fn alpha2_exact_equality_uses_official_source_operators() {
        let source = "판 <- 차림[차림[1, 2], 차림[3, 4]]. (판 == 판) 보여주기. (판 != 차림[차림[1, 2], 차림[4, 3]]) 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_equality.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else { panic!("output rows"); };
        assert_eq!(rows.len(), 2);
        for row in rows {
            let RuntimeValue::Map(row) = row else { panic!("output row"); };
            assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
                (RuntimeValue::String(key), RuntimeValue::String(text)) if key == "text" && text == "참")));
        }
    }

    #[test]
    fn alpha2_scene_elements_append_without_comparison_but_equality_rejects() {
        let prefix = "() 원그림:장면값:셈씨 := { 원 <- (중심=차림[0,0],반지름=1) 표준장면.원. 요소들 <- 차림[원]. 요소들 <- (요소들,원) 추가. 원그림 <- (요소들=요소들,가로범위=차림[-2,2],세로범위=차림[-2,2]) 표준장면.평면. }. 장면 := () 원그림.";
        let program = DdnProgram::from_v25_source(prefix, "scene_append.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let program = DdnProgram::from_v25_source(&format!("{prefix}(장면 == 장면) 보여주기."), "scene_equality.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("scene equality rejected");
        assert!(error.to_string().contains("E_SCENE_OPERATION_UNSUPPORTED"));
    }

    #[test]
    fn a3_rd136_timed_view_cannot_bypass_scene_comparison_guard() {
        let source = "원 := (중심=차림[0,0],반지름=1) 표준장면.원. 장면 := (요소들=차림[원],가로범위=차림[-2,2],세로범위=차림[-2,2]) 표준장면.평면. 보기 := (장면=장면,시간=() 표준시간.공통) 표준보기.장면. (보기 == 보기) 보여주기.";
        let program=DdnProgram::from_v25_source(source,"timed-scene-guard.ddn").unwrap();
        let mut runner=DdnRunner::new(program,"매마디");
        let error=runner.run_update(&NuriWorld::new(),&empty_input(),&HashMap::new()).err().expect("timed scene must still reject comparison");
        assert!(error.to_string().contains("E_SCENE_OPERATION_UNSUPPORTED"),"{error}");
    }

    #[test]
    fn alpha2_nested_selection_rebuilds_only_the_named_root() {
        let source = "판 <- 차림[차림[0, 0], 차림[0, 0]]. 이전 <- 판. 행 <- 1. 열 <- 0. 판[행][열] <- 7. (이전[1][0]) 보여주기. (판[1][0]) 보여주기. (판[0][0]) 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_nested_write.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).unwrap();
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else { panic!("output rows"); };
        assert_eq!(rows.len(), 3);
        for (row, expected) in rows.iter().zip(["0", "7", "0"]) {
            let RuntimeValue::Map(row) = row else { panic!("output row map"); };
            assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
                (RuntimeValue::String(key), RuntimeValue::String(text)) if key == "text" && text == expected)));
        }
    }

    #[test]
    fn vc13_sum_original_negative_frontdoor_contracts(){
        for (name,source,code) in [("vc08_duplicate.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $예. }.
}
"#,"E_SUM"),
("vc08_unknown.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$미지.
}
"#,"E_SUM_BRANCH_UNKNOWN"),
("vc08_match_duplicate.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 선택.$예이면 { }. 선택.$예이면 { }. 아니면 { }. }.
}
"#,"E_SUM_MATCH_BRANCH_DUPLICATE"),
("vc08_nonexhaustive.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 선택.$예이면 { }. }.
}
"#,"E_SUM_MATCH_EXHAUSTIVE_REQUIRED"),
("vc08_duplicate_default.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 선택.$예이면 { }. 아니면 { }. 아니면 { }. }.
}
"#,"E_SUM_MATCH"),
("vc08_nontrailing.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 아니면 { }. 선택.$예이면 { }. }.
}
"#,"E_SUM_MATCH"),
("vc08_nonsum.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 3. 값에 따라 { 선택.$예이면 { }. 아니면 { }. }.
}
"#,"E_SUM_MATCH_SUBJECT_TYPE"),
("vc08_crossowner.ddn",r#"매마디:움직씨 := {
첫째:고름씨 := { $예. }. 둘째:고름씨 := { $예. }. 값 <- 첫째.$예. 값에 따라 { 둘째.$예이면 { "금지" 보여주기. }. 아니면 { "금지" 보여주기. }. }.
}
"#,"E_SUM_MATCH_OWNER_MISMATCH"),
("vc08_payload_type.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$수(값="글").
}
"#,"E_RUNTIME_TYPE_MISMATCH"),
("vc08_bare.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- $예.
}
"#,"E_SUM_BRANCH_OWNER_REQUIRED"),
("vc08_split.ddn",r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$ 예.
}
"#,"$")]{
            let error=crate::runtime_surface::run_summary_from_supported_source_profile(source,name,Some(1),Some("v1-core-v25")).err().expect("original invalid owned category must reject without a summary");
            assert!(error.to_string().contains(code),"{name}: expected {code}: {error}");
        }
    }

    #[test]
    fn vc13_sum_payload_and_match_failures_preserve_the_world(){
        let source=r#"매마디:움직씨 := {
선택:고름씨 := { $수(받은값:수). }.
진행 <- 1. "게시 금지" 보여주기.
값 <- 선택.$수(받은값=3).
값에 따라 { 선택.$수(받은값)이면 { 목록 <- 차림[1]. 위치 <- 2. (목록[위치]) 보여주기. }. }.
}"#;
        let program=DdnProgram::from_v25_source(source,"sum-atomic.ddn").unwrap();
        let mut runner=DdnRunner::new(program,DEFAULT_UPDATE_NAME);
        let world=NuriWorld::new();let before=format!("{world:?}");
        let error=runner.run_update(&world,&empty_input(),&HashMap::new()).err().expect("late match failure must return no successful update");
        assert!(error.contains("CHARIM_INDEX_OUT_OF_RANGE"),"{error:?}");
        assert_eq!(format!("{world:?}"),before);
        assert!(crate::runtime_surface::run_summary_from_supported_source_profile(source,"sum-atomic.ddn",Some(1),Some("v1-core-v25")).is_err());
    }

    #[test]
    fn vc13_sum_is_not_product_or_numeric_selection(){
        // RD-348/357: OwnedBranchIdentity is not a field or ordinal slot.
        for access in ["값.값", "값[0]"] {
            let source=format!("매마디:움직씨 := {{ 선택:고름씨 := {{ $수(값:수). }}. 값 <- 선택.$수(값=3). ({access}) 보여주기. }}");
            assert!(crate::runtime_surface::run_summary_from_supported_source_profile(&source,"sum-role.ddn",Some(1),Some("v1-core-v25")).is_err(),"branch payload must not be decomposed through {access}");
        }
    }

    #[test]
    fn vc13_sum_pattern_binder_restores_its_lexical_value_and_type(){
        let cases:serde_json::Value=serde_json::from_str(include_str!("../../../tests/fixtures/vc13_sum_pattern_scope_cases.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let source=case["source"].as_str().unwrap();
            let result=crate::runtime_surface::run_summary_from_supported_source_profile(source,"sum-binder.ddn",Some(1),Some("v1-core-v25"));
            if let Some(code)=case["code"].as_str() {
                let error=result.err().unwrap_or_else(||panic!("{}: invalid payload pattern/type must reject",case["id"]));
                assert!(error.contains(code),"{}: {error}",case["id"]);
            } else {
                let summary=result.expect("a scoped pattern binder must not acquire a world write or leak its type");
                assert_eq!(summary["output_display"]["texts"],case["texts"],"{}",case["id"]);
            }
        }
    }

    #[test]
    fn vc13_sum_deep_payload_uses_existing_heap_owners(){
        std::thread::Builder::new().stack_size(128*1024).spawn(||{
            let mut value=Value::Bool(true);
            for _ in 0..20000 {value=Value::SumBranch{owner:"선택".into(),branch:"다음".into(),payload:BTreeMap::from([("값".into(),value)])};}
            let copy=value.clone();assert!(value==copy);
            let key=map_key_canon(&copy);assert!(key.starts_with("선택.$다음(값="));
            let display=value_to_display_text(&copy);assert!(display.starts_with("선택.$다음("));
            assert!(display.contains("참"));drop(copy);drop(value);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn vc13_sum_value_generation_and_current_match_execute_shared_frontdoor(){
        for (source,expected) in [(r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$수(값=3). 값 보여주기.
}
"#,vec!["선택.$수(3)"]),(r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(내용:수). }.
값 <- 선택.$수(내용=3). 값에 따라 { 선택.$예이면 { "예" 보여주기. }. 선택.$수(내용)이면 { 내용 보여주기. }. }.
}
"#,vec!["3"]),(r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$수(값=3). 값에 따라 { 선택.$예이면 { "예" 보여주기. }. 아니면 { "기본" 보여주기. }. }.
}
"#,vec!["기본"])]{
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(source,"sum-category.ddn",Some(1),Some("v1-core-v25")).expect("approved owned branch source executes");
            assert_eq!(summary["output_display"]["texts"],serde_json::json!(expected));
        }
    }
    #[test]
    fn beta1_target_owned_selection_map_read_and_persistent_nested_update() {
        // RD-353/355: strict selection, immutable replacement, retained alias.
        let source = "자료 <- (\"행\", 차림[2, 3]) 짝맞춤. 이전 <- 자료. 자료[\"행\"][1] <- 7. (이전[\"행\"][1]) 보여주기. (자료[\"행\"][1]) 보여주기.";
        let summary = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "map-selection.ddn", Some(1), Some("v1-core-v25"),
        ).expect("typed map selection runs through the product frontdoor");
        assert_eq!(summary["output_display"]["texts"], serde_json::json!(["3", "7"]));
    }

    #[test]
    fn beta1_list_range_runtime_bounds_are_half_open_and_replacement_is_persistent() {
        let source = "자료 <- 차림[10,20,30,40]. 처음 <- 1. 끝 <- 3. 이전 <- 자료. 조각 <- 자료[처음..끝]. 자료[처음..끝] <- 차림[7,8]. 조각 보여주기. 이전 보여주기. 자료 보여주기. (자료[4..4]) 보여주기. 자료[4..4] <- 자료[0..0]. 자료 보여주기.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(
            source,"list-range.ddn",Some(1),Some("v1-core-v25"),
        ).expect("runtime list bounds and exact replacement through product frontdoor");
        assert_eq!(summary["output_display"]["texts"],serde_json::json!([
            "차림[20, 30]","차림[10, 20, 30, 40]","차림[10, 7, 8, 40]","차림[]","차림[10, 7, 8, 40]",
        ]));
    }

    #[test]
    fn beta1_list_range_rejects_overflow_reversed_and_size_changing_replacement() {
        for (operation,code) in [
            ("(자료[1..5]) 보여주기.","E_SELECTION_RANGE_BOUNDS"),
            ("처음 <- 3. 끝 <- 1. (자료[처음..끝]) 보여주기.","E_SELECTION_RANGE_BOUNDS"),
            ("자료[1..3] <- 차림[7].","E_SELECTION_RANGE_REPLACEMENT"),
            ("자료[1..3] <- 차림[7,8,9].","E_SELECTION_RANGE_REPLACEMENT"),
            ("자료[1..3] <- 7.","E_SELECTION_RANGE_REPLACEMENT"),
            ("자료[4..4] <- 차림[7].","E_SELECTION_RANGE_REPLACEMENT"),
        ] {
            let source=format!("자료 <- 차림[10,20,30,40]. {operation}");
            let error=crate::runtime_surface::run_summary_from_supported_source_profile(
                &source,"list-range-invalid.ddn",Some(1),Some("v1-core-v25"),
            ).err().expect("invalid range must reject without publishing a summary");
            assert!(error.to_string().contains(code),"{operation}: {error}");
        }
    }

    #[test]
    fn beta1_list_range_nested_root_rebuild_keeps_alias_and_tensor_static_contract() {
        let source="자료 <- (\"행\",차림[10,20,30]) 짝맞춤. 이전 <- 자료. 처음 <- 1. 끝 <- 3. 자료[\"행\"][처음..끝] <- 차림[7,8]. (이전[\"행\"]) 보여주기. (자료[\"행\"]) 보여주기.";
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile(
            source,"nested-range.ddn",Some(1),Some("v1-core-v25"),
        ).unwrap();
        assert_eq!(summary["output_display"]["texts"],serde_json::json!(["차림[10, 20, 30]","차림[10, 7, 8]"]));
        let tensor="판 <- (차림=차림[10,20,30],원소씨=셈수,모양=(3,)) 텐서로. 처음 <- 0. 끝 <- 2. (판[처음..끝]) 보여주기.";
        let error=crate::runtime_surface::run_summary_from_supported_source_profile(
            tensor,"tensor-dynamic-range.ddn",Some(1),Some("v1-core-v25"),
        ).err().expect("runtime list allowance cannot relax RD388 Tensor bounds");
        assert!(error.to_string().contains("E_TENSOR_STATIC_RANGE"),"{error}");
    }

    #[test]
    fn beta1_target_owned_selection_rejects_missing_without_insertion() {
        for operation in ["(자료[\"없음\"]) 보여주기.", "자료[\"없음\"] <- 7.",
            "자료[\"행\"][\"없음\"] <- 7."] {
            let source = format!("자료 <- (\"행\", (\"칸\", 2) 짝맞춤) 짝맞춤. {operation}");
            let program = DdnProgram::from_v25_source(&source, "map-missing.ddn").unwrap();
            let mut runner = DdnRunner::new(program, "매마디");
            let before = NuriWorld::new();
            let error = runner.run_update(&before, &empty_input(), &HashMap::new())
                .err().expect("missing key cannot insert or return None");
            assert!(error.to_string().contains("E_SELECTION_KEY_MISSING"), "{error}");
        }
    }

    #[test]
    fn beta1_target_owned_selection_does_not_change_optional_map_provider() {
        let source = "자료 <- (\"칸\", 2) 짝맞춤. (((자료, \"없음\") 짝맞춤.값) 없는가) 보여주기. 자료 <- (자료, \"새칸\", 7) 짝맞춤.바꾼값. (자료[\"새칸\"]) 보여주기.";
        let summary = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "map-provider.ddn", Some(1), Some("v1-core-v25"),
        ).expect("explicit existing provider remains independent of strict selection");
        assert_eq!(summary["output_display"]["texts"], serde_json::json!(["참", "7"]));
    }

    #[test]
    fn beta1_target_owned_selection_text_does_not_fallback_to_codepoints() {
        assert_eq!(unicode_segmentation::UNICODE_VERSION, (17, 0, 0));
        assert_eq!(unicode_normalization::UNICODE_VERSION, (17, 0, 0));
        for (text, expected) in [
            ("👨‍👩‍👧‍👦끝", "👨‍👩‍👧‍👦"),
            ("👍🏽끝", "👍🏽"),
            ("🇰🇷끝", "🇰🇷"),
            ("e\u{301}끝", "e\u{301}"), // 글 값의 결합문자 bytes를 바꾸지 않는다.
            ("가끝", "가"),
        ] {
            let source = format!("글값 <- \"{text}\". 위치 <- 0. (글값[위치]) 보여주기. (글값[1]) 보여주기.");
            let summary = crate::runtime_surface::run_summary_from_supported_source_profile(
                &source, "text-selection.ddn", Some(1), Some("v1-core-v25"),
            ).expect("글 선택은 확장 글자 경계를 사용해야 한다");
            assert_eq!(summary["output_display"]["texts"], serde_json::json!([expected, "끝"]));
        }
    }

    #[test]
    fn beta1_target_owned_selection_text_rejects_invalid_positions() {
        for source in [
            "글값 <- \"👨‍👩‍👧‍👦\". (글값[1]) 보여주기.",
            "글값 <- \"\". (글값[0]) 보여주기.",
            "글값 <- \"가\". 위치 <- -1. (글값[위치]) 보여주기.",
            "글값 <- \"가\". 위치 <- 0.5. (글값[위치]) 보여주기.",
        ] {
            let error = crate::runtime_surface::run_summary_from_supported_source_profile(
                source, "text-selection-bad.ddn", Some(1), Some("v1-core-v25"),
            ).err().expect("범위 밖이나 정수가 아닌 위치는 거부해야 한다");
            assert!(error.to_string().contains("E_SELECTION_TEXT_"), "{error}");
        }
    }

    #[test]
    fn beta1_text_selection_replacement_preserves_graphemes_and_immutable_alias() {
        for value in ["👨‍👩‍👧‍👦", "👍🏽", "🇰🇷", "e\u{301}"] {
            let source=format!("원본 <- \"앞가끝\". 글값 <- 원본. 글값[1] <- \"{value}\". 원본 보여주기. 글값 보여주기. (글값[1]) 보여주기.");
            let summary=crate::runtime_surface::run_summary_from_supported_source_profile(
                &source,"text-replacement.ddn",Some(1),Some("v1-core-v25")).unwrap();
            assert_eq!(summary["output_display"]["texts"],serde_json::json!(["앞가끝",format!("앞{value}끝"),value]));
        }
    }

    #[test]
    fn beta1_text_selection_replacement_rejects_insert_delete_and_neighbour_merge() {
        for (source,code) in [
            ("글값 <- \"앞가끝\". 글값[1] <- \"\".","E_SELECTION_TEXT_REPLACEMENT"),
            ("글값 <- \"앞가끝\". 글값[1] <- \"두글\".","E_SELECTION_TEXT_REPLACEMENT"),
            ("글값 <- \"앞가끝\". 글값[3] <- \"나\".","E_SELECTION_TEXT_BOUNDS"),
            ("글값 <- \"ab\". 글값[1] <- \"\u{301}\".","E_SELECTION_TEXT_REPLACEMENT"),
        ] {
            let error=crate::runtime_surface::run_summary_from_supported_source_profile(
                source,"text-replacement-bad.ddn",Some(1),Some("v1-core-v25")).err().expect("replacement must reject");
            assert!(error.to_string().contains(code),"{error}");
        }
    }

    #[test]
    fn alpha2_nested_selection_rejects_every_missing_level() {
        // Admit a well-typed, in-bounds authored statement first. Inject an
        // invalid integer index only into this private evaluator fixture, so
        // runtime bounds checks are tested independently of static rejection.
        for (indices, index) in [("[쪽][0]",2),("[0][쪽]",2),
                                 ("[쪽][0]",-1),("[0][쪽]",-1)] {
            let source = format!("쪽 <- 0. 판 <- 차림[차림[\"가\",\"나\"],차림[\"다\",\"라\"]]. 판{indices} <- \"새\".");
            let program = DdnProgram::from_v25_source(&source, "alpha2_nested_bounds.ddn").unwrap();
            let runner = DdnRunner::new(program, "매마디");
            let world=NuriWorld::new();let defaults=HashMap::new();
            let mut context=EvalContext::new(&runner.program,&world,&defaults,None,
                InputState::new(0,0),0,0,None);
            context.tuple_type_plan=Rc::clone(&runner.tuple_type_plan);
            let mut locals=HashMap::new();
            for statement in &runner.program.program.root_stmts[..2] {
                context.eval_stmt(&mut locals,statement).unwrap();
            }
            assert!(matches!(context.get_resource("쪽"),Some(Value::Fixed64(_))),
                "private legacy numeric fixture must keep its original resource lane");
            context.set_resource("쪽",Value::Fixed64(Fixed64::from_i64(index))).unwrap();
            let before=format!("{:?}",context.patch_ops);
            let error=context.eval_stmt(&mut locals,&runner.program.program.root_stmts[2])
                .err().expect("missing existing selection rejected");
            assert!(error.to_string().contains("INDEX_OUT_OF_RANGE")
                || error.to_string().contains("음수")
                || error.to_string().contains("인덱스는 0 이상이어야 합니다"), "{indices}: {error}");
            assert_eq!(format!("{:?}",context.patch_ops),before,"failed replacement must not stage a partial board");
        }
        // Original invalid raw sources must still fail without publication;
        // static rejection is not misreported as runtime execution evidence.
        for indices in ["[2][0]","[0][2]","[-1][0]","[0][-1]"] {
            let source=format!("판 <- 차림[차림[0, 0], 차림[0, 0]]. 판{indices} <- 7.");
            let world=NuriWorld::new();let before=format!("{world:?}");
            let rejected=DdnProgram::from_v25_source(&source,"negative-literal.ddn")
                .and_then(|program|DdnRunner::new(program,"매마디").run_update(&world,&empty_input(),&HashMap::new()));
            let error=rejected.err().expect("invalid authored selection must not publish success");
            assert!(error.contains("INDEX_OUT_OF_RANGE") || error.contains("음수")
                || error.contains("인덱스는 0 이상이어야 합니다")
                || (error.contains("E_TYPE_MISMATCH") && error.contains("수 종류의 정적 공급")),
                "unrelated rejection must not satisfy the bounds regression: {indices}: {error}");
            assert_eq!(format!("{world:?}"),before);
        }
    }

    #[test]
    fn alpha2_typed_constructor_errors_keep_the_authored_call_location() {
        for newline in ["\n", "\r\n"] {
            let body = "기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸.\n짜임 := (관계들=차림[기본,기본]) 표준보개.짜임.";
            let source = format!("{newline}{newline}{}", body.replace('\n', newline));
            let program = DdnProgram::from_v25_source(&source, "한글_진단.ddn").unwrap();
            let mut runner = DdnRunner::new(program, "매마디");
            let error = runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new())
                .err().expect("duplicate primary rejected");
            let (code, message, span) = error.diagnostic_parts();
            assert_eq!(code, "E_RUNTIME_EXECUTION", "existing classification preserved");
            assert!(message.contains("E_BOGAE_PRIMARY_COUNT"));
            assert!(message.contains("(4:"), "{message}");
            let (start,end) = span.expect("structured source span");
            assert!(source[start..end].contains("표준보개.짜임"));
        }
    }

    #[test]
    fn alpha2_nested_typed_constructor_error_keeps_inner_location() {
        let source = "\n\n그림 := (장면=((요소들=차림[\n  (중심=차림[0,0], 반지름=0) 표준장면.원\n], 가로범위=차림[-1,1], 세로범위=차림[-1,1]) 표준장면.평면)) 표준보기.장면.";
        let program = DdnProgram::from_v25_source(source, "중첩.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .err().expect("nonpositive radius rejected");
        let (_,message,span)=error.diagnostic_parts();
        assert!(message.contains("E_SCENE_RADIUS"));
        assert!(message.contains("(4:"), "{message}");
        let (start,end)=span.unwrap();
        assert!(source[start..end].contains("표준장면.원"));
        assert!(!source[start..end].contains("표준장면.평면"));
    }

    #[test]
    fn alpha2_composition_rejects_primary_count_and_nonflat_carriers() {
        let prefix = "보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. ";
        let empty_source = format!("{prefix}짜임 := (관계들=차림[]) 표준보개.짜임.");
        let error = DdnProgram::from_v25_source(&empty_source, "alpha2_empty_composition.ddn")
            .err().expect("untyped empty list rejected by canonical parser");
        assert!(error.contains("E_COLLECTION_CONSTRUCTOR_ELEMENT_TYPE_REQUIRED"));
        for (relations, code) in [
            ("차림[기본, 기본]", "E_BOGAE_PRIMARY_COUNT"),
            ("차림[차림[기본]]", "E_BOGAE_UNSUPPORTED"),
            ("차림[\"으뜸\"]", "E_BOGAE_UNSUPPORTED"),
            ("기본", "E_BOGAE_UNSUPPORTED"),
        ] {
            let source = format!("{prefix}짜임 := (관계들={relations}) 표준보개.짜임.");
            let program = DdnProgram::from_v25_source(&source, "alpha2_invalid_composition.ddn").unwrap();
            let mut runner = DdnRunner::new(program, "매마디");
            let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("invalid composition rejected");
            assert!(error.to_string().contains(code), "{relations}: {error}");
        }
    }

    #[test]
    fn alpha2_composition_identity_operations_remain_unresolved() {
        let prefix = "보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임. ";
        for operation in [
            "(짜임 == 짜임) 보여주기.",
            "결과 := (짜임, 짜임) 모음.",
            "결과 := (짜임, 1) 짝맞춤.",
            "결과 := (차림[짜임], 짜임) 찾아보기.",
            "결과 := (관계들=차림[짜임]) 표준보개.짜임.",
        ] {
            let source = format!("{prefix}{operation}");
            let program = DdnProgram::from_v25_source(&source, "alpha2_unresolved_composition.ddn").unwrap();
            let mut runner = DdnRunner::new(program, "매마디");
            let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()).err().expect("unresolved composition rejected");
            assert!(error.to_string().contains("정체 해소 전")
                || error.to_string().contains("E_BOGAE_UNSUPPORTED"), "{operation}: {error}");
        }
    }

    #[test]
    fn alpha2_relation_guard_counts_repeated_visits_and_map_keys() {
        let value = Value::List(vec![Value::Bool(false); 64]);
        let mut meter = WorkCostMeter::default();
        for _ in 0..3 {
            assert!(!contains_unresolved_bogae_relation(&value, &mut meter).unwrap());
        }
        assert_eq!(meter.receipt().counters.collection_tensor_element_visits, 195);
        let map = Value::Map(BTreeMap::from([("key".to_string(), MapEntry {
            key: Value::List(vec![Value::Bool(true)]),
            value: Value::Bool(false),
        })]));
        assert!(!contains_unresolved_bogae_relation(&map, &mut meter).unwrap());
        assert_eq!(meter.receipt().counters.collection_tensor_element_visits, 199);
    }

    #[test]
    fn alpha2_relation_guard_is_iterative_for_deep_values() {
        let mut value = Value::Bool(false);
        for _ in 0..4096 {
            value = Value::List(vec![value]);
        }
        let mut meter = WorkCostMeter::default();
        assert!(!contains_unresolved_bogae_relation(&value, &mut meter).unwrap());
        assert_eq!(meter.receipt().counters.collection_tensor_element_visits, 4097);
        // 검사뿐 아니라 시험 값 폐기도 Rust 재귀를 사용하지 않는다.
        drop(value);
    }

    #[test]
    fn alpha2_relation_guard_propagates_meter_overflow() {
        let mut counters = crate::work_cost::WorkCostCounters::default();
        counters.work_cost = u64::MAX;
        let mut meter = WorkCostMeter::from_receipt_parts(
            counters, StandardResourceLimitsV1::CANDIDATE, false);
        let error = contains_unresolved_bogae_relation(&Value::Bool(false), &mut meter)
            .err().expect("guard must not ignore accounting failure");
        assert!(error.to_string().contains("E_WORK_COST_OVERFLOW_CANDIDATE"));
    }

    #[test]
    fn alpha2_unresolved_relations_do_not_merge_as_collection_keys() {
        let prefix = "보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. ";
        for operation in [
            "결과 := (기본, 기본) 모음.",
            "결과 := (기본, 1) 짝맞춤.",
            "결과 := (차림[기본], 기본) 들어있나.",
            "결과 := (차림[기본], 기본) 찾아보기.",
            "결과 := (차림[기본], 차림[기본]) 모음.",
        ] {
            let source = format!("{prefix}{operation}");
            let program = DdnProgram::from_v25_source(&source, "alpha2_relation_collection.ddn")
                .expect("official source parses");
            let mut runner = DdnRunner::new(program, "매마디");
            let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
                .err().expect("unresolved relation must not silently merge or search");
            assert!(error.to_string().contains("E_BOGAE_RELATION_UNRESOLVED"), "{operation}: {error}");
        }

        let source = format!("{prefix}관계들 := 차림[기본, 기본].");
        let program = DdnProgram::from_v25_source(&source, "alpha2_relation_list.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("ordered authored transport is not identity-based merging");
    }

    #[test]
    fn alpha2_unresolved_relation_comparison_fails_closed() {
        let source = "보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. (기본 == 기본) 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_relation_compare.ddn")
            .expect("official source frontdoor");
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .err().expect("unresolved identity is not view-content equality");
        assert!(error.to_string().contains("정체 해소 전"), "{error}");
    }

    #[test]
    fn alpha2_result_table_is_a_typed_non_world_view_binding() {
        let source = "보기:보기값 := () 표준보기.결과표. 보기 보여주기.";
        let program = DdnProgram::from_v25_source(source, "alpha2_result_table.ddn")
            .expect("official V25 frontdoor");
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("shared runtime constructor");
        let Some(RuntimeValue::List(rows)) = output.resources.get("output_log") else {
            panic!("show must publish a text row");
        };
        let RuntimeValue::Map(row) = &rows[0] else { panic!("output row"); };
        assert!(row.values().any(|entry| matches!((&entry.key, &entry.value),
            (RuntimeValue::String(key), RuntimeValue::String(text))
                if key == "text" && text == "<보기값:결과표>")));
        assert!(output.patch.ops.iter().all(|op| !matches!(op,
            PatchOp::SetResourceJson { tag, .. }
            | PatchOp::SetResourceFixed64 { tag, .. }
            | PatchOp::SetResourceValue { tag, .. } if tag == "보기")));
    }

    #[test]
    fn alpha2_view_type_rejects_a_plain_string_in_actual_source() {
        let program = DdnProgram::from_v25_source(
            "보기:보기값 := \"결과표\".", "alpha2_fake_view.ddn")
            .expect("typed source parses");
        let mut runner = DdnRunner::new(program, "매마디");
        let error = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .err().expect("plain text is not a view");
        assert!(error.to_string().contains("보기값"), "{error}");
    }

    #[test]
    fn v25_root_rebinding_type_mismatch_fails_before_any_run_output_is_published() {
        let source = "값 <- 0.\n값 <- \"글\".\n값 보여주기.\n";
        let program = DdnProgram::from_v25_source(source, "root_binding_type_mismatch.ddn")
            .expect("V25 root program parses");
        let mut runner = DdnRunner::new(program, "매마디");
        let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("root rebinding type mismatch must fail closed"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_ROOT_BINDING_TYPE_MISMATCH"));
    }

    #[test]
    fn pg_cli_text_and_number_addition_fails_typed_before_output_publication() {
        let source = "점수 <- 7.\n\"점수: \" + 점수 보여주기.\n";
        let world = NuriWorld::new();
        let before=world.state_hash();
        // Known operand kinds now fail at source admission, before creating
        // a VM; do not demand that invalid code reach runtime dispatch.
        let error=DdnProgram::from_v25_source(source,"pg_cli_text_number_addition.ddn")
            .and_then(|program|DdnRunner::new(program,"매마디")
                .run_update(&world,&empty_input(),&HashMap::new()).map_err(|error|error.to_string()))
            .err().expect("implicit text-number addition must fail closed");
        assert!(error.contains("E_CHECK_TYPE_MISMATCH"),"{error}");
        assert_eq!(world.state_hash(),before);
        assert!(world.get_resource_value("output_log").is_none());
        assert!(world.get_resource_value("점수").is_none());
    }

    #[test]
    fn pg_cli_text_addition_uses_the_existing_glro_conversion_owner() {
        let source = "점수 <- 7.\n\"점수: \" + (점수) 글로 보여주기.\n";
        let program = DdnProgram::from_v25_source(source, "pg_cli_explicit_text_conversion.ddn")
            .expect("the explicit current conversion surface parses");
        let mut runner = DdnRunner::new(program, "매마디");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("explicit 글로 conversion must reach normal string addition");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("the successful explicit conversion must publish one output entry");
        };
        let RuntimeValue::Map(entry) = entries.first().expect("one output entry") else {
            panic!("output entry must be a map");
        };
        let text = entry
            .values()
            .find(|field| matches!(&field.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|field| match &field.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("점수: 7"));
    }

    #[test]
    fn pg_cli_collection_constructor_executes_in_authored_order() {
        let source = r#"검산:움직씨 := {
    목록:(수)차림 := 차림[1, 2, 3].
    목록 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "collection_constructor.ddn")
            .expect("current collection source parses");
        let mut runner = DdnRunner::new(program, "검산");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("collection constructor executes");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("successful show must publish the output log");
        };
        let RuntimeValue::Map(entry) = entries.first().expect("one output entry") else {
            panic!("output entry must be a map");
        };
        let text = entry
            .values()
            .find(|field| matches!(&field.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|field| match &field.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("차림[1, 2, 3]"));
    }

    #[test]
    fn pg_cli_collection_constructor_type_failure_has_no_output_publication() {
        let source = r#"검산:움직씨 := {
    목록 <- 차림[1, "글"].
    목록 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "collection_constructor_mixed.ddn")
            .expect("mixed source reaches the typed runtime boundary");
        let mut runner = DdnRunner::new(program, "검산");
        let world = NuriWorld::new();
        let error = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("mixed element types must fail closed"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("차림") || error.to_string().contains("타입"));
        assert!(world.get_resource_value("output_log").is_none());
        assert!(world.get_resource_value("목록").is_none());
    }

    #[test]
    fn pg_cli_selection_place_rebinds_the_root_once() {
        let source = r#"검산:움직씨 := {
    목록:(수)차림 := 차림[1, 2, 3].
    목록[1] <- 9.
    목록 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "selection_place.ddn")
            .expect("current selection source parses");
        let mut runner = DdnRunner::new(program, "검산");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("indexed replacement executes");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("successful show must publish the output log");
        };
        let RuntimeValue::Map(entry) = entries.first().expect("one output entry") else {
            panic!("output entry must be a map");
        };
        let text = entry
            .values()
            .find(|field| matches!(&field.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|field| match &field.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("차림[1, 9, 3]"));
    }

    #[test]
    fn pg_cli_selection_place_failure_has_no_partial_root_or_output_publication() {
        let source = r#"검산:움직씨 := {
    목록:(수)차림 := 차림[1, 2, 3].
    목록[3] <- 9.
    목록 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "selection_place_range.ddn")
            .expect("current selection source parses");
        let mut runner = DdnRunner::new(program, "검산");
        let world = NuriWorld::new();
        let error = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("out-of-range indexed replacement must fail closed"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("CHARIM_INDEX_OUT_OF_RANGE"),
            "unexpected selection failure: {}",
            error
        );
        assert!(world.get_resource_value("목록").is_none());
        assert!(world.get_resource_value("output_log").is_none());
    }

    #[test]
    fn v25_typed_legacy_equals_fails_before_runtime_state_exists() {
        let script = "계산:움직씨 := {\n    값:수 = 1.\n}\n";
        let error = match DdnProgram::from_v25_source(script, "pc01_typed_legacy_equals.ddn") {
            Ok(_) => panic!("typed legacy equals must fail before a runtime program exists"),
            Err(error) => error,
        };
        assert!(error.contains("E_V25_DEFINITION_MIGRATION_REQUIRED"));
    }

    #[test]
    fn pg_cli_rd406_named_single_expression_callable_runs_and_checks_its_result_type() {
        let source = r#"자리번호 (가로:셈수, 세로:셈수 --> 셈수) := 세로 * 10 + 가로.

검산:움직씨 := {
    (4, 3) 자리번호 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "pg_cli_pure_callable.ddn")
            .expect("named callable source parses through the shared runtime frontdoor");
        let mut runner = DdnRunner::new(program, "검산");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("typed named callable executes");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("successful show must publish the output log");
        };
        let RuntimeValue::Map(entry) = entries.first().expect("one output entry") else {
            panic!("output entry must be a map");
        };
        let text = entry
            .values()
            .find(|field| matches!(&field.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|field| match &field.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("34"));

        let wrong_result = r#"자리번호 (가로:셈수 --> 셈수) := "글".
검산:움직씨 := { (4) 자리번호 보여주기. }
"#;
        let program = DdnProgram::from_v25_source(wrong_result, "pg_cli_pure_callable_bad.ddn")
            .expect("result type failure reaches the runtime boundary");
        let mut runner = DdnRunner::new(program, "검산");
        let world = NuriWorld::new();
        let error = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("wrong typed callable result must fail closed"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("자리번호 결과"));
        assert!(world.get_resource_value("output_log").is_none());
    }

    #[test]
    fn pg_cli_result_flow_updates_a_selection_only_after_the_candidate_is_ready() {
        let source = r#"검산:움직씨 := {
    판:(수)차림 := 차림[0, 0].
    1 -> 판[0].
    판 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(source, "pg_cli_result_flow.ddn")
            .expect("result-flow source parses through the shared runtime frontdoor");
        let mut runner = DdnRunner::new(program, "검산");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("a valid immutable selection replacement runs");
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("successful result-flow show must publish one output entry");
        };
        let RuntimeValue::Map(entry) = entries.first().expect("one output entry") else {
            panic!("output entry must be a map");
        };
        let text = entry
            .values()
            .find(|field| matches!(&field.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|field| match &field.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("차림[1, 0]"));

        let invalid = r#"검산:움직씨 := {
    판:(수)차림 := 차림[0].
    1 -> 판[2].
    판 보여주기.
}
"#;
        let program = DdnProgram::from_v25_source(invalid, "pg_cli_result_flow_bounds.ddn")
            .expect("invalid index is a runtime candidate failure, not parser lowering");
        let mut runner = DdnRunner::new(program, "검산");
        let world = NuriWorld::new();
        let error = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("out-of-range result-flow must fail closed"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("FATAL:CHARIM_INDEX_OUT_OF_RANGE"),
            "unexpected result-flow failure: {error}"
        );
        assert!(world.get_resource_value("output_log").is_none());
    }

    #[test]
    fn raw_ddn_simplify_rejects_definedness_losing_cancellation() {
        let source = r#"
정의역검산:움직씨 = {
  결과 <- ((#ascii) 수식{(x^2 - 1)/(x - 1)}) 정리하기.
  결과 보여주기.
}
"#;
        let program = DdnProgram::from_source(source, "definedness_containment.ddn")
            .expect("raw DDN fixture must parse");
        let mut runner = DdnRunner::new(program, "정의역검산");
        let world = NuriWorld::new();
        let error = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("정리하기가 전제 없이 정의역을 축소하면 안 됨"),
            Err(error) => error,
        };
        assert!(
            error.contains("E_SYMBOLIC_DEFINEDNESS_OBLIGATION_REQUIRED"),
            "unexpected raw DDN error: {}",
            error
        );
    }

    fn extract_fixed(resources: &HashMap<String, RuntimeValue>, key: &str) -> Fixed64 {
        match resources.get(key) {
            Some(RuntimeValue::Fixed64(value)) => *value,
            Some(other) => panic!("{} must be Fixed64, got {:?}", key, other),
            None => panic!("{} missing", key),
        }
    }

    fn foreach_fixture(body_source: &str) -> (DdnProgram, Stmt) {
        let source = format!(
            r#"
매틱:움직씨 = {{
  (항목) 대상에 대해 {{
    {body_source}
  }}.
}}
"#
        );
        let program = DdnProgram::from_source(&source, "foreach_scope_fixture.ddn").expect("parse");
        let seed = program.functions.get("매틱").expect("매틱 seed");
        let stmt = seed
            .body
            .as_ref()
            .expect("매틱 body")
            .stmts
            .first()
            .expect("foreach statement")
            .clone();
        assert!(matches!(stmt, Stmt::ForEach { .. }));
        (program, stmt)
    }

    fn foreach_error_fixture() -> (DdnProgram, Stmt) {
        foreach_fixture("없는값 보여주기.")
    }

    fn foreach_call_error_fixture() -> (DdnProgram, Stmt) {
        let source = r#"
실패:움직씨 = {
  없는값 보여주기.
}

매틱:움직씨 = {
  (항목) 대상에 대해 {
    () 실패.
  }.
}
"#;
        let program =
            DdnProgram::from_source(source, "foreach_scope_call_error.ddn").expect("parse");
        let stmt = program.functions["매틱"].body.as_ref().expect("body").stmts[0].clone();
        (program, stmt)
    }

    fn foreach_error_locals() -> HashMap<String, RuntimeValue> {
        HashMap::from([
            (
                "대상".to_string(),
                RuntimeValue::List(vec![
                    RuntimeValue::String("첫째".to_string()),
                    RuntimeValue::String("둘째".to_string()),
                ]),
            ),
            ("항목".to_string(), RuntimeValue::String("바깥".to_string())),
        ])
    }

    fn assignment_stmt_fixture(assignment: &str, function_body: &str) -> (DdnProgram, Stmt) {
        let source = format!(
            r#"
원천:셈씨 = {{
  {function_body}
}}.

매틱:움직씨 = {{
  {assignment}
}}.
"#
        );
        let program = DdnProgram::from_source(&source, "assignment_arrow_fixture.ddn")
            .expect("assignment fixture parse");
        let stmt = program.functions["매틱"]
            .body
            .as_ref()
            .expect("매틱 body")
            .stmts[0]
            .clone();
        assert!(matches!(stmt, Stmt::Mutate { .. }));
        (program, stmt)
    }

    #[test]
    fn bidirectional_assignment_has_same_shared_runtime_state() {
        let left_source = "채비 { 값:수 <- 0. }.\n값 <- 3.\n";
        let right_source = "채비 { 값:수 <- 0. }.\n3 -> 값.\n";
        let mut left = DdnRunner::new(
            DdnProgram::from_source(left_source, "assignment_left.ddn").expect("left parse"),
            "매틱",
        );
        let mut right = DdnRunner::new(
            DdnProgram::from_source(right_source, "assignment_right.ddn").expect("right parse"),
            "매틱",
        );
        let world = NuriWorld::new();
        let left_output = left
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("left run");
        let right_output = right
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("right run");
        assert_eq!(left_output.resources, right_output.resources);
        assert_eq!(
            extract_fixed(&right_output.resources, "값"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn right_assignment_evaluates_source_once_in_shared_runtime() {
        let (program, stmt) =
            assignment_stmt_fixture("() 원천 -> 값.", "횟수 <- 횟수 + 1.\n  3 돌려줘.");
        let world = NuriWorld::new();
        let defaults = HashMap::from([("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO))]);
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = HashMap::from([
            ("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO)),
            ("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO)),
        ]);
        ctx.eval_stmt(&mut locals, &stmt)
            .expect("right assignment run");
        assert_eq!(
            ctx.resources.get("횟수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            locals.get("값"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(3)))
        );
    }

    #[test]
    fn bidirectional_assignment_source_failure_is_atomic_in_shared_runtime() {
        for assignment in ["값 <- () 원천.", "() 원천 -> 값."] {
            let (program, stmt) =
                assignment_stmt_fixture(assignment, "횟수 <- 횟수 + 1.\n  없는값 돌려줘.");
            let world = NuriWorld::new();
            let defaults =
                HashMap::from([("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO))]);
            let mut ctx = EvalContext::new(
                &program,
                &world,
                &defaults,
                None,
                InputState::new(0, 0),
                0,
                0,
                None,
            );
            let mut locals = HashMap::from([
                ("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO)),
                ("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO)),
            ]);
            let err = match ctx.eval_stmt(&mut locals, &stmt) {
                Err(err) => err,
                Ok(_) => panic!("failing source must fail assignment"),
            };
            assert!(err.to_string().contains("없는값"), "{err}");
            assert!(!ctx.resources.contains_key("횟수"));
            assert!(ctx.patch_ops.is_empty());
            assert_eq!(
                locals.get("값"),
                Some(&RuntimeValue::Fixed64(Fixed64::ZERO))
            );
        }
    }

    #[test]
    fn foreach_body_error_restores_shadowed_binding_in_flow_runtime() {
        let (program, stmt) = foreach_error_fixture();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = foreach_error_locals();

        let err = match ctx.eval_stmt(&mut locals, &stmt) {
            Err(err) => err,
            Ok(_) => panic!("foreach body must preserve the original error"),
        };

        assert!(err.to_string().contains("없는값"), "{err}");
        assert_eq!(
            locals.get("항목"),
            Some(&RuntimeValue::String("바깥".to_string()))
        );
    }

    #[test]
    fn natural_iteration_current_item_runs_through_existing_foreach_runtime() {
        let source = r#"
(값:수) 누적:움직씨 = {
  합 <- 합 + 값.
}

매틱:움직씨 = {
  채비 {
    목록:(수)차림 <- (1, 2, 3) 차림.
    합:수 <- 0.
  }.
  목록마다 {
    () 누적하기.
  }.
}
"#;
        let program = DdnProgram::from_source(source, "natural_iteration_runtime.ddn")
            .unwrap_or_else(|err| {
                let prepared = crate::preprocess::preprocess_source_for_parse(source)
                    .unwrap_or_else(|pre_err| format!("preprocess failed: {pre_err}"));
                panic!("shared parser must lower natural iteration: {err}\n--- prepared ---\n{prepared}")
            });
        assert!(program.canonical_ddn().contains("(목록:수) 목록에 대해"));
        assert!(program.canonical_ddn().contains("목록:값 누적기."));
        assert!(!program.canonical_ddn().contains("목록마다"));

        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("natural iteration runtime");
        assert_eq!(extract_fixed(&output.resources, "합"), Fixed64::from_i64(6));
        assert!(matches!(
            output.resources.get("목록"),
            Some(RuntimeValue::List(values)) if values.len() == 3
        ));
    }

    #[test]
    fn foreach_iterable_expression_is_observably_evaluated_once() {
        let source = r#"
() 목록생성:셈씨 = {
  "ITERABLE_EVAL" 보여주기.
  평가횟수 <- 평가횟수 + 1.
  목록 돌려줘.
}

매틱:움직씨 = {
  채비 {
    평가횟수:수 <- 0.
    합:수 <- 0.
    목록:(수)차림 <- (1, 2, 3) 차림.
  }.
  (항목:수) () 목록생성하기에 대해 {
    합 <- 합 + 항목.
  }.
}
"#;
        let program = DdnProgram::from_source(source, "foreach_iterable_once.ddn")
            .expect("observable iterable expression");
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("foreach iterable execution");
        assert_eq!(
            extract_fixed(&output.resources, "평가횟수"),
            Fixed64::from_i64(1)
        );
        assert_eq!(extract_fixed(&output.resources, "합"), Fixed64::from_i64(6));
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("output_log must be list");
        };
        let marker_count = entries
            .iter()
            .filter(|entry| {
                let RuntimeValue::Map(map) = entry else {
                    return false;
                };
                map.values().any(|entry| {
                    matches!(&entry.key, RuntimeValue::String(key) if key == "text")
                        && matches!(&entry.value, RuntimeValue::String(value) if value == "ITERABLE_EVAL")
                })
            })
            .count();
        assert_eq!(marker_count, 1, "iterable expression must emit once");
    }

    #[test]
    fn natural_iteration_handles_zero_one_and_1001_items_without_a_second_runtime_path() {
        for count in [0_i64, 1, 1001] {
            let items = (1..=count)
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!(
                r#"
(값:수) 누적:움직씨 = {{
  합 <- 합 + 값.
}}

매틱:움직씨 = {{
  채비 {{
    목록:(수)차림 <- ({items}) 차림.
    합:수 <- 0.
  }}.
  목록마다 {{ () 누적하기. }}.
}}
"#
            );
            let program = DdnProgram::from_source(&source, "natural_iteration_cardinality.ddn")
                .unwrap_or_else(|err| panic!("count={count}: {err}"));
            let mut runner = DdnRunner::new(program, "매틱");
            let output = runner
                .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
                .unwrap_or_else(|err| panic!("count={count}: {err}"));
            let expected = count * (count + 1) / 2;
            assert_eq!(
                extract_fixed(&output.resources, "합"),
                Fixed64::from_i64(expected),
                "count={count}"
            );
            assert!(matches!(
                output.resources.get("목록"),
                Some(RuntimeValue::List(values)) if values.len() == count as usize
            ));
        }
    }

    #[test]
    fn foreach_body_error_restores_shadowed_binding_in_value_runtime() {
        let (program, stmt) = foreach_error_fixture();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = foreach_error_locals();

        let err = match ctx.eval_stmt_for_value(&mut locals, &stmt) {
            Err(err) => err,
            Ok(_) => panic!("foreach body must preserve the original error"),
        };

        assert!(err.to_string().contains("없는값"), "{err}");
        assert_eq!(
            locals.get("항목"),
            Some(&RuntimeValue::String("바깥".to_string()))
        );
    }

    #[test]
    fn foreach_body_call_error_restores_shadowed_binding_in_both_eval_paths() {
        for value_path in [false, true] {
            let (program, stmt) = foreach_call_error_fixture();
            let world = NuriWorld::new();
            let defaults = HashMap::new();
            let mut ctx = EvalContext::new(
                &program,
                &world,
                &defaults,
                None,
                InputState::new(0, 0),
                0,
                0,
                None,
            );
            let mut locals = foreach_error_locals();

            let err = if value_path {
                match ctx.eval_stmt_for_value(&mut locals, &stmt) {
                    Err(err) => err,
                    Ok(_) => panic!("call error must propagate from value path"),
                }
            } else {
                match ctx.eval_stmt(&mut locals, &stmt) {
                    Err(err) => err,
                    Ok(_) => panic!("call error must propagate from flow path"),
                }
            };

            assert!(err.to_string().contains("없는값"), "{err}");
            assert_eq!(
                locals.get("항목"),
                Some(&RuntimeValue::String("바깥".to_string()))
            );
        }
    }

    #[test]
    fn failed_foreach_run_keeps_world_hash_and_returns_no_committed_output() {
        let (program, _) = foreach_call_error_fixture();
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let before_hash = world.state_hash();
        let defaults = HashMap::from([
            (
                "대상".to_string(),
                RuntimeValue::List(vec![RuntimeValue::String("첫째".to_string())]),
            ),
            ("항목".to_string(), RuntimeValue::String("바깥".to_string())),
        ]);

        let first = runner.run_update(&world, &empty_input(), &defaults);
        assert!(
            first.is_err(),
            "failed foreach must not return a patch or receipt"
        );
        assert_eq!(world.state_hash(), before_hash);

        let second = runner.run_update(&world, &empty_input(), &defaults);
        assert!(
            second.is_err(),
            "rerun must fail again without stale success"
        );
        assert_eq!(world.state_hash(), before_hash);
    }

    #[test]
    fn foreach_scope_restores_or_removes_binding_for_container_matrix() {
        let containers = vec![
            ("empty_list", RuntimeValue::List(Vec::new())),
            (
                "single_list",
                RuntimeValue::List(vec![RuntimeValue::String("하나".to_string())]),
            ),
            (
                "multi_list",
                RuntimeValue::List(vec![
                    RuntimeValue::String("하나".to_string()),
                    RuntimeValue::String("둘".to_string()),
                ]),
            ),
            (
                "set",
                RuntimeValue::Set(BTreeMap::from([
                    ("a".to_string(), RuntimeValue::String("하나".to_string())),
                    ("b".to_string(), RuntimeValue::String("둘".to_string())),
                ])),
            ),
            (
                "map",
                RuntimeValue::Map(BTreeMap::from([
                    (
                        "a".to_string(),
                        ddonirang_lang::runtime::MapEntry {
                            key: RuntimeValue::String("a".to_string()),
                            value: RuntimeValue::String("하나".to_string()),
                        },
                    ),
                    (
                        "b".to_string(),
                        ddonirang_lang::runtime::MapEntry {
                            key: RuntimeValue::String("b".to_string()),
                            value: RuntimeValue::String("둘".to_string()),
                        },
                    ),
                ])),
            ),
        ];

        for (name, container) in containers {
            for had_outer_binding in [false, true] {
                let (program, stmt) = foreach_fixture("항목 보여주기.");
                let world = NuriWorld::new();
                let defaults = HashMap::new();
                let mut ctx = EvalContext::new(
                    &program,
                    &world,
                    &defaults,
                    None,
                    InputState::new(0, 0),
                    0,
                    0,
                    None,
                );
                let mut locals = HashMap::from([("대상".to_string(), container.clone())]);
                if had_outer_binding {
                    locals.insert("항목".to_string(), RuntimeValue::String("바깥".to_string()));
                }

                match ctx.eval_stmt(&mut locals, &stmt) {
                    Ok(FlowControl::Continue) => {}
                    _ => panic!("{name}: normal foreach must continue"),
                }
                if had_outer_binding {
                    assert_eq!(
                        locals.get("항목"),
                        Some(&RuntimeValue::String("바깥".to_string())),
                        "{name}"
                    );
                } else {
                    assert!(!locals.contains_key("항목"), "{name}");
                }
            }
        }
    }

    #[test]
    fn foreach_scope_restores_binding_for_flow_matrix() {
        for (name, body) in [
            ("normal", "항목 보여주기."),
            ("continue", "건너뛰기."),
            ("break", "멈추기."),
            ("return", "항목 되돌림."),
        ] {
            let (program, stmt) = foreach_fixture(body);
            let world = NuriWorld::new();
            let defaults = HashMap::new();
            let mut ctx = EvalContext::new(
                &program,
                &world,
                &defaults,
                None,
                InputState::new(0, 0),
                0,
                0,
                None,
            );
            let mut locals = foreach_error_locals();

            let result = ctx.eval_stmt(&mut locals, &stmt);
            assert!(result.is_ok(), "{name}: foreach must preserve flow");
            assert_eq!(
                locals.get("항목"),
                Some(&RuntimeValue::String("바깥".to_string())),
                "{name}"
            );
        }
    }

    #[test]
    fn foreach_scope_removes_new_binding_after_body_error() {
        let (program, stmt) = foreach_error_fixture();
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = HashMap::from([(
            "대상".to_string(),
            RuntimeValue::List(vec![RuntimeValue::String("첫째".to_string())]),
        )]);

        assert!(ctx.eval_stmt(&mut locals, &stmt).is_err());
        assert!(!locals.contains_key("항목"));
    }

    #[test]
    fn foreach_scope_does_not_create_binding_for_invalid_iterable() {
        let (program, stmt) = foreach_fixture("항목 보여주기.");
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = HashMap::from([(
            "대상".to_string(),
            RuntimeValue::String("순회불가".to_string()),
        )]);

        assert!(ctx.eval_stmt(&mut locals, &stmt).is_err());
        assert!(!locals.contains_key("항목"));
    }

    #[test]
    fn foreach_scope_restores_binding_after_abort() {
        let (program, stmt) =
            foreach_fixture("{ 거짓 }인것 바탕으로(물림) 아니면 {\n      없음.\n    }.");
        let world = NuriWorld::new();
        let defaults = HashMap::new();
        let mut ctx = EvalContext::new(
            &program,
            &world,
            &defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut locals = foreach_error_locals();

        assert!(ctx.eval_stmt(&mut locals, &stmt).is_ok());
        assert!(ctx.aborted);
        assert_eq!(
            locals.get("항목"),
            Some(&RuntimeValue::String("바깥".to_string()))
        );
    }

    #[test]
    fn nested_foreach_error_restores_same_name_and_distinct_outer_bindings() {
        for (outer_name, inner_name) in [("항목", "항목"), ("바깥항목", "안쪽항목")] {
            let source = format!(
                r#"
매틱:움직씨 = {{
  ({outer_name}) 바깥대상에 대해 {{
    ({inner_name}) 안쪽대상에 대해 {{
      없는값 보여주기.
    }}.
  }}.
}}
"#
            );
            let program =
                DdnProgram::from_source(&source, "nested_foreach_scope_error.ddn").expect("parse");
            let stmt = program.functions["매틱"].body.as_ref().expect("body").stmts[0].clone();
            let world = NuriWorld::new();
            let defaults = HashMap::new();
            let mut ctx = EvalContext::new(
                &program,
                &world,
                &defaults,
                None,
                InputState::new(0, 0),
                0,
                0,
                None,
            );
            let mut locals = HashMap::from([
                (
                    "바깥대상".to_string(),
                    RuntimeValue::List(vec![RuntimeValue::String("바깥현재".to_string())]),
                ),
                (
                    "안쪽대상".to_string(),
                    RuntimeValue::List(vec![RuntimeValue::String("안쪽현재".to_string())]),
                ),
                (
                    outer_name.to_string(),
                    RuntimeValue::String("바깥원본".to_string()),
                ),
                (
                    inner_name.to_string(),
                    RuntimeValue::String("안쪽원본".to_string()),
                ),
            ]);
            if outer_name == inner_name {
                locals.insert(
                    outer_name.to_string(),
                    RuntimeValue::String("공유원본".to_string()),
                );
            }

            assert!(ctx.eval_stmt(&mut locals, &stmt).is_err());
            if outer_name == inner_name {
                assert_eq!(
                    locals.get(outer_name),
                    Some(&RuntimeValue::String("공유원본".to_string()))
                );
            } else {
                assert_eq!(
                    locals.get(outer_name),
                    Some(&RuntimeValue::String("바깥원본".to_string()))
                );
                assert_eq!(
                    locals.get(inner_name),
                    Some(&RuntimeValue::String("안쪽원본".to_string()))
                );
            }
        }
    }

    fn linear_inequality_test_condition(body: &str, compare: &str, boundary: i64) -> RuntimeValue {
        let mut fields = BTreeMap::new();
        fields.insert(
            "식".to_string(),
            RuntimeValue::Formula(Formula {
                raw: body.to_string(),
                dialect: FormulaDialect::Ascii,
                explicit_tag: true,
            }),
        );
        fields.insert(
            "비교".to_string(),
            RuntimeValue::String(compare.to_string()),
        );
        fields.insert(
            "경계".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(boundary)),
        );
        RuntimeValue::Pack(fields)
    }

    #[test]
    fn double_arrow_is_rendezvous_and_never_dispatches_legacy_async_handler() {
        let script = r#"
채비 {
  받은:글 <- "".
}.

(값:수) 첫알림:알림씨 = {
  없음.
}

관제탑:임자 = {
  첫알림을 받으면 {
    받은 <- "ok".
  }.
}

매틱:움직씨 = {
  ((값=1) 첫알림) ~~> 관제탑.
}
"#;
        let program = DdnProgram::from_source(script, "imja_signal.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("받은".to_string(), RuntimeValue::String(String::new()));
        let error = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("rendezvous runtime remains product-open"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_RENDEZVOUS_PRODUCT_OPEN"));
    }

    #[test]
    fn successor_async_send_uses_exactly_one_endpoint_route() {
        let script = r#"
채비 {
  받은:글 <- "".
}.

(값:수) 첫알림:알림씨 = { 없음. }

관제탑:임자 = {
  (받은알림:첫알림)를 받으면 { 받은 <- "async-ok". }.
}

매틱:움직씨 = {
  (철수)의 ((값=1) 첫알림) ~> 관제탑.
}
"#;
        let program = DdnProgram::from_source(script, "successor_async_one.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("받은".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("받은"),
            Some(&RuntimeValue::String("async-ok".to_string()))
        );
    }

    #[test]
    fn a3_rd293_signal_payload_checks_declared_pin_before_reaction() {
        // A3-P4: typed alert admission is independent of whether a provider reads it.
        let source = r#"
(값:수) 첫알림:알림씨 := { 없음. }.
관제탑:임자 := {
  (받은:첫알림) 첫반응:움직씨 {첫알림을 받으면} := { 성공 <- 7. }.
}.
매틱:움직씨 := { (철수)의 ((값=3) 첫알림) ~> 관제탑. }.
"#;
        let run = |text: &str| {
            let program = DdnProgram::from_v25_source(text, "알림씨_받는자리.ddn")?;
            DdnRunner::new(program, "매틱").run_update(
                &NuriWorld::new(), &empty_input(), &HashMap::new())
        };
        assert_eq!(run(source).unwrap().resources["성공"], Value::Fixed64(Fixed64::from_i64(7)));
        for wrong in ["\"틀린씨\"", "참"] {
            let invalid = source.replace("값=3", &format!("값={wrong}"));
            let error = run(&invalid).err().expect("wrong payload must fail before dispatch");
            assert!(error.contains("E_TYPE_MISMATCH"), "{error}");
        }
        let text_pin = source.replace("값:수", "값:글").replace("값=3", "값=\"정상\"");
        assert!(run(&text_pin).is_ok());
        assert!(run(&text_pin.replace("값=\"정상\"", "값=3")).is_err());
    }

    #[test]
    fn a3_rd445_authored_providers_share_snapshot_and_fail_on_conflict() {
        // A3-P4 provisional header placement; actual V25 product parse/run.
        let source=r#"
채비 { 합계:수 := 2. 관찰값:수 := 0. 수신값:수 := 0. }.
(값:수) 첫알림:알림씨 := { 없음. }.
관제탑:임자 := {
  (받은:첫알림) 첫반응:움직씨 {첫알림을 받으면} := { 합계 <- 합계 + 받은.값. }.
  (받은:첫알림) 둘반응:움직씨 {첫알림을 받으면} := { 관찰값 <- 합계 + 10. }.
}.
매틱:움직씨 := { (철수)의 ((값=1) 첫알림) ~> 관제탑. }.
"#;
        let defaults=HashMap::from([("합계".into(),Value::Fixed64(Fixed64::from_i64(2))),
            ("관찰값".into(),Value::Fixed64(Fixed64::from_i64(0)))]);
        let run=|text:&str| {
            let program=DdnProgram::from_v25_source(text,"반응.ddn").unwrap();
            DdnRunner::new(program,"매틱").run_update(&NuriWorld::new(),&empty_input(),&defaults)
        };
        let output=run(source).unwrap();
        assert_eq!(output.resources["합계"],Value::Fixed64(Fixed64::from_i64(3)));
        assert_eq!(output.resources["관찰값"],Value::Fixed64(Fixed64::from_i64(12)),"second provider observed first provider's write");
        let repeated=source.replace("합계 <- 합계 + 받은.값.","합계 <- 합계 + 0. 합계 <- 합계 + 받은.값.");
        assert_eq!(run(&repeated).unwrap().resources,output.resources,"one provider's sequential writes are not cross-provider conflicts");
        let conflict=source.replace("관찰값 <- 합계 + 10","합계 <- 합계 + 10");
        assert!(run(&conflict).err().unwrap().contains("E_REACTION_PATCH_CONFLICT"));
        let failure=source.replace("관찰값 <- 합계 + 10","관찰값 <- 1 / 0");
        assert!(run(&failure).is_err());
        let combined=source.replace("관제탑:임자 := {", "관제탑:임자 := { (받은:첫알림)를 받으면 { 수신값 <- 합계 + 20. }.");
        assert_eq!(run(&combined).unwrap().resources["수신값"],Value::Fixed64(Fixed64::from_i64(22)),"consuming route must see the same settled snapshot as providers");
        let ambiguous=combined.replace("수신값 <- 합계 + 20. }.","수신값 <- 합계 + 20. }. (받은:첫알림)를 받으면 { 없음. }.");
        assert!(run(&ambiguous).err().unwrap().contains("E_ASYNC_ENDPOINT_AMBIGUOUS"));
        for mutation in ["받은 <- 없음.", "받은.값 <- 7.", "받는이 <- \"다른이\"."] {
            let immutable=source.replace("합계 <- 합계 + 받은.값.",mutation);
            assert!(run(&immutable).is_err(), "RD-293 immutable payload/receiver accepted: {mutation}");
        }
        let receiver=source.replace("관찰값 <- 합계 + 10.","수신이름 <- 받는이.");
        assert_eq!(run(&receiver).unwrap().resources["수신이름"],Value::String("관제탑".into()));
    }

    #[test]
    fn successor_async_send_fails_for_zero_and_multiple_routes() {
        let zero = r#"
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
관제탑:임자 = { (받은알림:둘알림)를 받으면 { 없음. }. }
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
        let program = DdnProgram::from_source(zero, "successor_async_zero.ddn").expect("parse");
        let error = match DdnRunner::new(program, "매틱").run_update(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("zero route must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_ASYNC_ENDPOINT_UNHANDLED"));

        let multiple = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 { 없음. }.
  (받은알림:첫알림)를 받으면 { 없음. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
        let program =
            DdnProgram::from_source(multiple, "successor_async_multiple.ddn").expect("parse");
        let error = match DdnRunner::new(program, "매틱").run_update(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("multiple routes must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_ASYNC_ENDPOINT_AMBIGUOUS"));
    }

    #[test]
    fn successor_async_nested_send_runs_at_queue_tail_without_reentry() {
        let script = r#"
채비 { 순서:수 <- 0. }.
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 {
    순서 <- 순서 * 10 + 1.
    ((값=0) 둘알림) ~> 제.
    순서 <- 순서 * 10 + 2.
  }.
  (받은알림:둘알림)를 받으면 { 순서 <- 순서 * 10 + 3. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
        let program = DdnProgram::from_source(script, "successor_async_tail.ddn").expect("parse");
        let mut defaults = HashMap::new();
        defaults.insert(
            "순서".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = DdnRunner::new(program, "매틱")
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("순서"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(123)))
        );
    }

    #[test]
    fn successor_async_bounded_full_wait_resumes_fifo_without_drop() {
        let script = r#"
채비 { 순서:수 <- 0. }.
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
(값:수) 셋알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 {
    순서 <- 순서 * 10 + 1.
    ((값=0) 둘알림) ~> 제.
    ((값=0) 셋알림) ~> 제.
    순서 <- 순서 * 10 + 2.
  }.
  (받은알림:둘알림)를 받으면 { 순서 <- 순서 * 10 + 3. }.
  (받은알림:셋알림)를 받으면 { 순서 <- 순서 * 10 + 4. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
        let program = DdnProgram::from_source(script, "successor_async_wait.ddn").expect("parse");
        let mut defaults = HashMap::new();
        defaults.insert(
            "순서".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = DdnRunner::new(program, "매틱")
            .with_async_queue_capacity(1)
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("bounded WAIT resumes");
        assert_eq!(
            output.resources.get("순서"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1234)))
        );
    }

    #[test]
    fn successor_sender_filter_matches_exact_identity_and_typed_payload() {
        let script = r#"
채비 { 받은:수 <- 0. }.
관제탑:임자 = {
  철수에게서 (받은수:수)를 받으면 { 받은 <- 받은수. }.
}
매틱:움직씨 = { (철수)의 7 ~> 관제탑. }
"#;
        let program = DdnProgram::from_source(script, "successor_sender_exact.ddn").expect("parse");
        let mut defaults = HashMap::new();
        defaults.insert(
            "받은".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = DdnRunner::new(program, "매틱")
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("exact sender route");
        assert_eq!(
            output.resources.get("받은"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(7)))
        );
    }

    #[test]
    fn successor_sender_filter_mismatch_is_unhandled() {
        let script = r#"
관제탑:임자 = {
  철수에게서 (받은수:수)를 받으면 { 없음. }.
}
매틱:움직씨 = { (영희)의 7 ~> 관제탑. }
"#;
        let program =
            DdnProgram::from_source(script, "successor_sender_mismatch.ddn").expect("parse");
        let error = match DdnRunner::new(program, "매틱").run_update(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("sender mismatch must not deliver"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_ASYNC_ENDPOINT_UNHANDLED"));
    }

    #[test]
    fn successor_sender_and_message_type_predicates_select_exactly_one() {
        let script = r#"
채비 { 받은:수 <- 0. }.
관제탑:임자 = {
  철수에게서 (받은수:수)를 받으면 { 받은 <- 1. }.
  영희에게서 (받은수:수)를 받으면 { 받은 <- 2. }.
  철수에게서 (받은글:글)를 받으면 { 받은 <- 3. }.
}
매틱:움직씨 = { (철수)의 7 ~> 관제탑. }
"#;
        let program =
            DdnProgram::from_source(script, "successor_sender_type_matrix.ddn").expect("parse");
        let mut defaults = HashMap::new();
        defaults.insert(
            "받은".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = DdnRunner::new(program, "매틱")
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("one semantic sender/type route");
        assert_eq!(
            output.resources.get("받은"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
    }

    #[test]
    fn successor_owner_context_supplies_sender_identity() {
        let script = r#"
채비 { 받은:수 <- 0. }.
중계소:임자 = {
  철수에게서 (받은수:수)를 받으면 { 받은수 ~> 관제탑. }.
}
관제탑:임자 = {
  중계소에게서 (받은수:수)를 받으면 { 받은 <- 받은수. }.
}
매틱:움직씨 = { (철수)의 7 ~> 중계소. }
"#;
        let program = DdnProgram::from_source(script, "successor_owner_sender.ddn").expect("parse");
        let mut defaults = HashMap::new();
        defaults.insert(
            "받은".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = DdnRunner::new(program, "매틱")
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("owner context sender");
        assert_eq!(
            output.resources.get("받은"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(7)))
        );
    }

    #[test]
    fn successor_historical_receive_hook_is_not_auto_migrated() {
        let script = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = { 첫알림을 받으면 { 없음. }. }
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
        let program = DdnProgram::from_source(script, "successor_no_historical_auto_migration.ddn")
            .expect("parse");
        let error = match DdnRunner::new(program, "매틱").run_update(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("historical receive hook must not become a successor route"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("E_ASYNC_ENDPOINT_UNHANDLED"));
    }

    #[test]
    fn successor_top_level_send_without_sender_identity_fails_closed() {
        let script = r#"
관제탑:임자 = { (받은수:수)를 받으면 { 없음. }. }
매틱:움직씨 = { 7 ~> 관제탑. }
"#;
        let program =
            DdnProgram::from_source(script, "successor_sender_missing.ddn").expect("parse");
        let error = match DdnRunner::new(program, "매틱").run_update(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("missing sender identity must fail closed"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("E_ASYNC_SENDER_IDENTITY_UNRESOLVED_CANDIDATE"));
    }

    #[test]
    fn top_level_param_override_drives_start_hook() {
        let source = r#"
채비 {
  __wasm_start_once: 수 <- 0.
  초기속도x: 수 <- 8.
}.

(시작)할때 {
  vx <- 초기속도x.
}.

(매마디)마다 {
  x <- vx.
}.
"#;
        let program = DdnProgram::from_source(source, "param_override_start.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut overrides = BTreeMap::new();
        overrides.insert(
            "초기속도x".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(19)),
        );
        let output = runner
            .run_update_with_param_overrides(
                &NuriWorld::new(),
                &empty_input(),
                &HashMap::new(),
                &overrides,
            )
            .expect("run with setup override");
        assert_eq!(
            extract_fixed(&output.resources, "초기속도x"),
            Fixed64::from_i64(19)
        );
        assert_eq!(
            extract_fixed(&output.resources, "vx"),
            Fixed64::from_i64(19)
        );
        assert_eq!(extract_fixed(&output.resources, "x"), Fixed64::from_i64(19));
    }

    #[test]
    fn top_level_setup_initializes_once_across_committed_ticks() {
        let source = r#"
채비 {
  t: 수 <- 0.
}.

(매마디)마다 {
  t <- t + 1.
  t 보여주기.
}.
"#;
        let program = DdnProgram::from_source(source, "setup_once.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        let mut observed = Vec::new();

        for tick_id in 0..3 {
            let mut input = empty_input();
            input.tick_id = tick_id;
            let output = runner
                .run_update(world.world(), &input, &HashMap::new())
                .expect("run update");
            observed.push(extract_fixed(&output.resources, "t"));
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
        }

        assert_eq!(
            observed,
            vec![
                Fixed64::from_i64(1),
                Fixed64::from_i64(2),
                Fixed64::from_i64(3),
            ]
        );
    }

    #[test]
    fn v25_pure_callable_bindings_do_not_leak_to_same_named_world_resources() {
        let source = r#"
(값:수) 지역계산:수:셈씨 := {
    값 <- 값 + 1.
    임시 <- 20.
    임시 <- 임시 + 1.
    지역계산 <- 값 + 임시.
}.
(시작)할때 { 값 <- 100. 임시 <- 300. }.
(매마디)마다 { 결과 <- (값=2) 지역계산. }.
"#;
        let program = DdnProgram::from_v25_source(source, "pure_local_resource.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        for tick_id in 0..3 {
            let mut input = empty_input(); input.tick_id = tick_id;
            let output = runner.run_update(world.world(), &input, &HashMap::new()).expect("run");
            assert_eq!(extract_fixed(&output.resources, "결과"), Fixed64::from_i64(24));
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
            let resources = crate::runtime_surface::serialize_world_resources(world.world());
            assert_eq!(resources["fixed64"]["값"], "100");
            assert_eq!(resources["fixed64"]["임시"], "300");
        }
    }

    #[test]
    fn v25_resultless_pure_callable_bindings_remain_local() {
        let source = r#"
(값:수) 지역계산:셈씨 := { 값 <- 값 + 1. 임시 <- 20. }.
(시작)할때 { 값 <- 100. 임시 <- 300. }.
(매마디)마다 { (값=2) 지역계산. }.
"#;
        let program = DdnProgram::from_v25_source(source, "resultless_local.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        for tick_id in 0..3 {
            let mut input = empty_input(); input.tick_id = tick_id;
            let output = runner.run_update(world.world(), &input, &HashMap::new()).expect("run");
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
            let resources = crate::runtime_surface::serialize_world_resources(world.world());
            assert_eq!(resources["fixed64"]["값"], "100");
            assert_eq!(resources["fixed64"]["임시"], "300");
        }
    }

    #[test]
    fn v25_result_flow_to_ordinary_pin_is_not_world_alias() {
        let source = r#"
(판:(수)차림) 지역계산:수:셈씨 := { 7 -> 판[0]. 지역계산 <- 판[0]. }.
(시작)할때 { 판 <- 차림[100,300]. }.
(매마디)마다 { 결과 <- (판=차림[2,3]) 지역계산. }.
"#;
        let program = DdnProgram::from_v25_source(source, "resultflow_local.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        for tick_id in 0..3 {
            let mut input = empty_input(); input.tick_id = tick_id;
            let output = runner.run_update(world.world(), &input, &HashMap::new()).expect("run");
            assert_eq!(extract_fixed(&output.resources, "결과"), Fixed64::from_i64(7));
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
            let resources = crate::runtime_surface::serialize_world_resources(world.world());
            assert_eq!(resources["value_json"]["판"][0].as_f64(), Some(100.0));
            assert_eq!(resources["value_json"]["판"][1].as_f64(), Some(300.0));
        }
    }

    #[test]
    fn v25_definition_binding_initializes_top_level_state_for_periodic_hook() {
        let source = r#"
채비 {
  t: 수 := 0.
}.

(매마디)마다 {
  t <- t + 1.
}.
"#;
        let program = DdnProgram::from_v25_source(source, "v25_definition_hook.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut world = DetNuri::new();
        let mut observed = Vec::new();

        for tick_id in 0..3 {
            let mut input = empty_input();
            input.tick_id = tick_id;
            let output = runner
                .run_update(world.world(), &input, &HashMap::new())
                .expect("run periodic hook");
            observed.push(extract_fixed(&output.resources, "t"));
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            world.apply_patch(&output.patch, tick_id, &mut sink);
        }

        assert_eq!(
            observed,
            vec![
                Fixed64::from_i64(1),
                Fixed64::from_i64(2),
                Fixed64::from_i64(3),
            ]
        );
    }

    fn contract_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L0-CONTRACT-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn assertion_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L1-ASSERT-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn state_transition_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L1-STATE-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn numeric_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED => Some(event),
                _ => None,
            })
            .collect()
    }

    fn numeric_factor_route_summary_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY => Some(event),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn from_source_rejects_legacy_header_frontdoor() {
        let source = "#이름: legacy\nx <- 1.\n";
        let err = match DdnProgram::from_source(source, "legacy_header.ddn") {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_source_rejects_legacy_boim_surface_frontdoor() {
        let source = "보임 {\n  x: 1.\n}.\n";
        let err = match DdnProgram::from_source(source, "boim.ddn") {
            Ok(_) => panic!("legacy must fail"),
            Err(err) => err,
        };
        assert!(err.contains("E_BOIM_LEGACY_REMOVED"), "{err}");
    }

    #[test]
    fn setting_madi_is_extracted_from_program() {
        let source = r#"
설정 {
  제목: 마디수_시험.
  마디수: 3.
}.

채비 {
  t: 수 <- 0.
}.
"#;
        let program = DdnProgram::from_source(source, "<test>").expect("program");
        assert_eq!(program.configured_madi(), Some(3));
    }

    #[test]
    fn setting_madi_rejects_non_positive_value() {
        let source = r#"
설정 {
  마디수: 0.
}.

채비 {
  t: 수 <- 0.
}.
"#;
        let err = match DdnProgram::from_source(source, "<test>") {
            Ok(_) => panic!("bad madi must fail"),
            Err(err) => err,
        };
        assert!(err.contains("E_SETTING_MADI_BAD_VALUE"), "{err}");
    }

    #[test]
    fn from_source_accepts_decl_item_maegim_suffix_frontdoor() {
        let source = r#"
매틱:움직씨 = {
  채비 {
    데이터길이:수 <- (12) 매김 {
      범위: 4..40.
      간격: 1.
    }.
  }.
}.
"#;
        let _program = DdnProgram::from_source(source, "decl_item_maegim.ddn")
            .expect("decl item maegim must parse");
    }

    #[test]
    fn from_source_accepts_legacy_bogae_draw_shorthand_frontdoor() {
        let source = r#"
채비 {
  데이터길이:수 <- (12) 매김 {
    범위: 4..40.
    간격: 1.
  }.
}.

(시작)할때 {
  t <- 0.
}.

(매마디)마다 {
  t <- t + 1.
}.

보개로 그려.
"#;
        let _program = DdnProgram::from_source(source, "legacy_bogae_draw_frontdoor.ddn")
            .expect("legacy bogae draw shorthand must parse");
    }

    #[test]
    fn keyboard_alias_resources_track_hold_press_release() {
        let script = r#"
매틱:움직씨 = {
}.
"#;
        let program = DdnProgram::from_source(script, "keyboard_alias_flags.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults = HashMap::new();

        let mut first = empty_input();
        first.keys_pressed = KEY_A;
        first.last_key_name = "ArrowLeft".to_string();
        let first_out = runner
            .run_update(&world, &first, &defaults)
            .expect("first run update");
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.누르고있음.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.눌림.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.뗌.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "입력상태.키_눌림.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.눌림.Z키"),
            Fixed64::from_i64(0)
        );

        let mut second = empty_input();
        second.keys_pressed = 0;
        let second_out = runner
            .run_update(&world, &second, &defaults)
            .expect("second run update");
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.누르고있음.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.눌림.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.뗌.왼쪽화살표"),
            Fixed64::from_i64(1)
        );

        let mut third = empty_input();
        third.keys_pressed = INPUT_KEY_Z;
        third.last_key_name = "KeyZ".to_string();
        let third_out = runner
            .run_update(&world, &third, &defaults)
            .expect("third run update");
        assert_eq!(
            extract_fixed(&third_out.resources, "샘.키보드.눌림.Z키"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&third_out.resources, "입력상태.키_눌림.Z키"),
            Fixed64::from_i64(1)
        );
    }

    #[test]
    fn bogae_omitted_numeric_defaults_are_exact_fixed64_raw_values() {
        assert_eq!(BOGAE_DEFAULT_STROKE_WIDTH.raw_i64(), 85_899_345);
        assert_eq!(BOGAE_DEFAULT_CIRCLE_RADIUS.raw_i64(), 343_597_383);
        assert_eq!(BOGAE_DEFAULT_POINT_SIZE.raw_i64(), 193_273_528);
        assert_eq!(BOGAE_DEFAULT_RECT_WIDTH.raw_i64(), 858_993_459);
        assert_eq!(BOGAE_DEFAULT_RECT_HEIGHT.raw_i64(), 515_396_075);
        assert_eq!(BOGAE_DEFAULT_TEXT_SIZE.raw_i64(), 343_597_383);
    }

    #[test]
    fn bogae_omitted_shape_fields_publish_exact_fixed64_defaults() {
        let script = r##"
매틱:움직씨 = {
    모양: {
        선(0, 0, 1, 1).
        원(0, 0).
        점(0, 0).
        사각형(0, 0).
        글(0, 0, "x").
    }.
}
"##;
        let program = DdnProgram::from_source(script, "bogae_defaults.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("run omitted-default shapes");
        let Some(RuntimeValue::List(items)) = output.resources.get("보개_그림판_목록") else {
            panic!("drawlist must be present");
        };
        let value = |item: &RuntimeValue, key: &str| {
            let RuntimeValue::Map(map) = item else {
                panic!("draw item must be a map");
            };
            map.values()
                .find(|entry| matches!(&entry.key, RuntimeValue::String(name) if name == key))
                .map(|entry| entry.value.clone())
                .expect("draw field")
        };
        assert_eq!(value(&items[0], "width"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(85_899_345)));
        assert_eq!(value(&items[1], "r"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(343_597_383)));
        assert_eq!(value(&items[2], "size"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(193_273_528)));
        assert_eq!(value(&items[3], "w"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(858_993_459)));
        assert_eq!(value(&items[3], "h"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(515_396_075)));
        assert_eq!(value(&items[4], "size"), RuntimeValue::Fixed64(Fixed64::from_raw_i64(343_597_383)));
    }

    #[test]
    fn bogae_shape_block_emits_drawlist() {
        let script = r##"
매틱:움직씨 = {
    bob_x <- 0.5.
    bob_y <- -0.8.
    모양: {
        선(0, 0, bob_x, bob_y, 색="#9ca3af", 굵기=0.02).
        polyline(0, 0, 0.25, 0.2, 0.4, -0.1, 색="#22c55e", 굵기=0.03, 층=2, 그룹="pendulum.path").
        polygon(-0.1, -0.05, 0.1, -0.05, 0.0, 0.16, 채움색="#f59e0b", 선색="#7c2d12", 굵기=0.01, 레이어=5, group="pendulum.body").
        사각형(-0.2, -0.1, 0.4, 0.2, 채움색="#1e293b", 선색="#38bdf8", 굵기=0.01).
        글(-0.1, 0.25, "추", 크기=0.09, 색="#f8fafc").
        원(bob_x, bob_y, r=0.08, 색="#38bdf8", 선색="#0ea5e9", 굵기=0.02).
        점(0, 0, 크기=0.045, 색="#f59e0b").
    }.
}
"##;
        let program = DdnProgram::from_source(script, "bogae_shape.ddn").expect("parse");
        let seed = program.functions.get("매틱").expect("매틱 seed");
        let body = seed.body.as_ref().expect("매틱 body");
        let Stmt::MetaBlock { entries, .. } = &body.stmts[2] else {
            panic!("모양 meta block expected");
        };
        assert_eq!(entries.len(), 7, "entries={entries:?}");
        let world = NuriWorld::new();
        let preview_defaults = HashMap::new();
        let mut preview_ctx = EvalContext::new(
            &program,
            &world,
            &preview_defaults,
            None,
            InputState::new(0, 0),
            0,
            0,
            None,
        );
        let mut preview_locals = HashMap::new();
        preview_locals.insert(
            "bob_x".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_f64_lossy(0.5)),
        );
        preview_locals.insert(
            "bob_y".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_f64_lossy(-0.8)),
        );
        assert!(
            preview_ctx
                .bogae_entry_to_draw_item(&mut preview_locals, &entries[1])
                .is_some(),
            "polyline entry must render: {}",
            entries[1]
        );
        assert!(
            preview_ctx
                .bogae_entry_to_draw_item(&mut preview_locals, &entries[2])
                .is_some(),
            "polygon entry must render: {}",
            entries[2]
        );
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        let drawlist = output.resources.get("보개_그림판_목록");
        let Some(RuntimeValue::List(items)) = drawlist else {
            panic!("보개_그림판_목록 must be list, got {:?}", drawlist);
        };
        assert_eq!(items.len(), 7);

        let kinds: Vec<String> = items
            .iter()
            .map(|item| {
                let RuntimeValue::Map(map) = item else {
                    panic!("도형은 map 이어야 합니다");
                };
                map.values()
                    .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "kind"))
                    .and_then(|entry| match &entry.value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .expect("kind string")
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["line", "polyline", "polygon", "rect", "text", "circle", "point"]
        );

        let RuntimeValue::Map(polyline) = &items[1] else {
            panic!("꺾은선은 map 이어야 합니다");
        };
        let polyline_points = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "points"))
            .map(|entry| entry.value.clone());
        let Some(RuntimeValue::List(poly_points)) = &polyline_points else {
            panic!("꺾은선 points must be list");
        };
        assert_eq!(poly_points.len(), 3);
        let polyline_layer = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "layer_index"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polyline_layer,
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        let polyline_group = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "group_id"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polyline_group,
            Some(RuntimeValue::String("pendulum.path".to_string()))
        );

        let RuntimeValue::Map(polygon) = &items[2] else {
            panic!("다각형은 map 이어야 합니다");
        };
        let polygon_fill = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "fill"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_fill,
            Some(RuntimeValue::String("#f59e0b".to_string()))
        );
        let polygon_layer = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "layer_index"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_layer,
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(5)))
        );
        let polygon_group = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "group_id"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_group,
            Some(RuntimeValue::String("pendulum.body".to_string()))
        );

        let RuntimeValue::Map(rect) = &items[3] else {
            panic!("사각형은 map 이어야 합니다");
        };
        let rect_fill = rect
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "fill"))
            .map(|entry| entry.value.clone());
        assert_eq!(rect_fill, Some(RuntimeValue::String("#1e293b".to_string())));

        let RuntimeValue::Map(text) = &items[4] else {
            panic!("글은 map 이어야 합니다");
        };
        let text_value = text
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "text"))
            .map(|entry| entry.value.clone());
        assert_eq!(text_value, Some(RuntimeValue::String("추".to_string())));
    }

    #[test]
    fn bogae_shape_helpers_parse_polyline_and_polygon() {
        let polyline = parse_bogae_shape_call(
            r##"polyline(0, 0, 0.25, 0.2, 0.4, -0.1, 색="#22c55e", 굵기=0.03)"##,
        )
        .expect("polyline parse");
        assert_eq!(polyline.positional.len(), 6);
        assert!(polyline.named.contains_key("색"));

        let polygon = parse_bogae_shape_call(
            r##"polygon(-0.1, -0.05, 0.1, -0.05, 0.0, 0.16, 채움색="#f59e0b", 선색="#7c2d12", 굵기=0.01)"##,
        )
        .expect("polygon parse");
        assert_eq!(polygon.positional.len(), 6);
        assert!(polygon.named.contains_key("채움색"));
    }

    #[test]
    fn map_dot_access_existing_key_runs() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    값 <- m.a.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
    }

    #[test]
    fn map_dot_access_missing_key_is_fatal() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    값 <- m.b.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("missing key must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_FIELD_MISSING") && err.contains("`b`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn map_optional_lookup_returns_none_when_missing() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    있음 <- (m, "a") 찾기.
    없음인가 <- ((m, "b") 찾기) 없는가.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("있음".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("없음인가".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "있음"),
            Fixed64::from_i64(1)
        );
        match output.resources.get("없음인가") {
            Some(RuntimeValue::Bool(true)) => {}
            other => panic!("없음인가 must be true, got {:?}", other),
        }
    }

    #[test]
    fn optional_presence_checks_do_not_use_payload_truthiness() {
        // RD-088 observes absence, not the truthiness of the contained value.
        let source = r#"
매틱:움직씨 = {
    m <- ("zero", 0, "false", 거짓) 짝맞춤.
    a <- ((m, "zero") 찾기) 있는가.
    b <- ((m, "false") 찾기) 있는가.
    c <- ((m, "missing") 찾기) 있는가.
    d <- ((m, "zero") 찾기) 없는가.
    e <- ((m, "missing") 찾기) 없는가.
}
"#;
        let program = DdnProgram::from_source(source, "presence.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("presence checks");
        for (name, expected) in [("a", true), ("b", true), ("c", false), ("d", false), ("e", true)] {
            assert!(matches!(output.resources.get(name), Some(RuntimeValue::Bool(value)) if *value == expected), "{name}");
        }
    }

    #[test]
    fn gini_and_quantile_builtins_support_aliases() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (1, 2, 3, 4) 차림.
    지니1 <- (값목록) 지니.
    지니2 <- (값목록) 지니계수.
    분위1 <- (값목록, 0.75, "선형보간") 분위수.
    분위2 <- (값목록, 0.75, "최근순위") 백분위수.
}
"#;
        let program = DdnProgram::from_source(script, "stats.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["지니1", "지니2", "분위1", "분위2"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "지니1"),
            Fixed64::from_f64_lossy(0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "지니2"),
            Fixed64::from_f64_lossy(0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "분위1"),
            Fixed64::from_f64_lossy(3.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "분위2"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn gini_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (1073741824, 1) 차림.
    결과 <- (값목록) 지니.
}
"#;
        let program = DdnProgram::from_source(script, "gini_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("gini overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn quantile_rejects_unknown_mode() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (1, 2, 3, 4) 차림.
    분위 <- (값목록, 0.5, "linear") 분위수.
}
"#;
        let program = DdnProgram::from_source(script, "stats_bad_mode.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("unknown mode must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("분위수 mode는 선형보간 또는 최근순위여야 합니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn linear_quantile_extreme_interpolation_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (-2147483647, 2147483647.99999999976716935634613037109375) 차림.
    결과 <- (값목록, 0.5, "선형보간") 분위수.
}
"#;
        let program = DdnProgram::from_source(script, "quantile_extreme.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("unrepresentable interpolation must reject"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn stdlib_l1_integrator_and_interpolation_builtins() {
        let script = r#"
매틱:움직씨 = {
    값1 <- (1, 2, 0.5) 적분.오일러.
    쌍값 <- (1, 0, -1, 0.25) 적분.반암시적오일러.
    x2 <- (쌍값, 0) 차림.값.
    v2 <- (쌍값, 1) 차림.값.
    보간1 <- (0, 10, 0.25) 보간.선형.
    보간2 <- (0, 10, 0.25, 0.5) 보간.계단.
    보간3 <- (0, 10, 0.75, 0.5) 보간.계단.
}
"#;
        let program = DdnProgram::from_source(script, "stdlib_l1_integrators.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["값1", "x2", "v2", "보간1", "보간2", "보간3"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "값1"),
            Fixed64::from_i64(2)
        );
        assert_eq!(
            extract_fixed(&output.resources, "x2"),
            Fixed64::from_f64_lossy(0.9375)
        );
        assert_eq!(
            extract_fixed(&output.resources, "v2"),
            Fixed64::from_f64_lossy(-0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "보간1"),
            Fixed64::from_f64_lossy(2.5)
        );
        assert_eq!(extract_fixed(&output.resources, "보간2"), Fixed64::ZERO);
        assert_eq!(
            extract_fixed(&output.resources, "보간3"),
            Fixed64::from_i64(10)
        );
    }

    #[test]
    fn stdlib_l1_filter_builtins() {
        let script = r#"
매틱:움직씨 = {
    창0 <- () 차림.
    결과1 <- (창0, 1) 필터.이동평균.
    창1 <- (결과1, 0) 차림.값.
    평균1 <- (결과1, 1) 차림.값.
    결과2 <- (창1, 2) 필터.이동평균.
    창2 <- (결과2, 0) 차림.값.
    평균2 <- (결과2, 1) 차림.값.
    결과3 <- (창2, 3) 필터.이동평균.
    창3 <- (결과3, 0) 차림.값.
    평균3 <- (결과3, 1) 차림.값.
    결과4 <- (창3, 4) 필터.이동평균.
    평균4 <- (결과4, 1) 차림.값.
    y1 <- (0, 1, 0.5) 필터.지수평활.
    y2 <- (y1, 1, 0.5) 필터.지수평활.
}
"#;
        let program = DdnProgram::from_source(script, "stdlib_l1_filters.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["평균1", "평균2", "평균3", "평균4", "y1", "y2"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "평균1"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균2"),
            Fixed64::from_f64_lossy(1.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균3"),
            Fixed64::from_i64(2)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균4"),
            Fixed64::from_f64_lossy(2.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "y1"),
            Fixed64::from_f64_lossy(0.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "y2"),
            Fixed64::from_f64_lossy(0.75)
        );
    }

    #[test]
    fn exponential_smoothing_preserves_fixed64_max_endpoint() {
        let script = r#"
매틱:움직씨 = {
    y <- (2147483647.99999999976716935634613037109375, 2147483647.99999999976716935634613037109375, 0.5) 필터.지수평활.
}
"#;
        let program = DdnProgram::from_source(script, "exponential_smoothing_max.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("representable endpoint must run");

        assert_eq!(extract_fixed(&output.resources, "y"), Fixed64::MAX);
    }

    #[test]
    fn exponential_smoothing_rounds_the_combined_weighted_value_once() {
        let script = r#"
매틱:움직씨 = {
    y <- (-0.000000004656612873077392578125, -0.00000000442378222942352294921875, 0.12500000023283064365386962890625) 필터.지수평활.
}
"#;
        let program = DdnProgram::from_source(script, "exponential_smoothing_tie.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("exact weighted blend must run");

        assert_eq!(
            extract_fixed(&output.resources, "y"),
            Fixed64::from_raw_i64(-20),
            "component-wise rounded products would produce -19 instead"
        );
    }

    #[test]
    fn moving_average_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    창 <- (1073741824) 차림.
    결과 <- (창, 1073741824) 필터.이동평균.
}
"#;
        let program = DdnProgram::from_source(script, "moving_average_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::List(Vec::new()));
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("moving-average overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn fixed64_min_abs_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    결과 <- (값) abs.
}
"#;
        let program = DdnProgram::from_source(script, "fixed64_min_abs.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::MIN));
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("Fixed64 minimum abs must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn fixed64_ceil_and_round_reject_unrepresentable_integer_results() {
        for operation in [fixed64_ceil, fixed64_round_even] {
            let error = operation(Fixed64::MAX)
                .expect_err("rounding a positive fractional maximum must reject");
            assert_eq!(error, ArithmeticFaultKind::Overflow);
        }

        assert_eq!(
            fixed64_ceil(Fixed64::from_raw_i64(Fixed64::ONE.raw_i64() + 1)),
            Ok(Fixed64::from_i64(2))
        );
        assert_eq!(
            fixed64_round_even(Fixed64::from_raw_i64(
                Fixed64::ONE.raw_i64() + (Fixed64::ONE.raw_i64() / 2),
            )),
            Ok(Fixed64::from_i64(2))
        );
    }

    #[test]
    fn fixed64_rounding_runtime_overflow_publishes_nothing() {
        for method in ["천장", "반올림"] {
            let script = format!(
                "매틱:움직씨 = {{\n    결과 <- (값) {method}.\n}}\n"
            );
            let program = DdnProgram::from_source(&script, "fixed64_rounding_overflow.ddn")
                .expect("parse rounding program");
            let mut runner = DdnRunner::new(program, "매틱");
            let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
            defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::MAX));
            defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

            let error = match runner.run_update_diagnostic(
                &NuriWorld::new(),
                &empty_input(),
                &defaults,
            ) {
                Err(error) => error,
                Ok(_) => panic!("rounding overflow must reject before publication"),
            };
            assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
        }
    }

    #[test]
    fn fixed64_powi_overflow_is_typed_and_does_not_saturate() {
        let error = unit_powi(
            UnitValue {
                value: Fixed64::MAX,
                dim: UnitDim::NONE,
            },
            2,
        )
        .expect_err("Fixed64 powi overflow must reject");
        assert!(matches!(error, EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow)));
    }

    #[test]
    fn fixed64_powi_runtime_overflow_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    결과 <- (값, 2) powi.
}
"#;
        let program = DdnProgram::from_source(script, "fixed64_powi_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::MAX));
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("Fixed64 powi overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn fixed64_infix_multiply_overflow_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    결과 <- 값 * 값.
}
"#;
        let program = DdnProgram::from_source(script, "fixed64_infix_mul_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::MAX));
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
        {
            Ok(_) => panic!("Fixed64 infix multiply overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn fixed64_infix_add_sub_overflow_publish_nothing() {
        for (operator, lhs, rhs) in [
            ("+", Fixed64::MAX, Fixed64::ONE),
            ("-", Fixed64::MIN, Fixed64::ONE),
        ] {
            let script = format!(
                "매틱:움직씨 = {{\n    결과 <- 값 {operator} 다른값.\n}}\n"
            );
            let program = DdnProgram::from_source(&script, "fixed64_infix_add_sub_overflow.ddn")
                .expect("parse");
            let mut runner = DdnRunner::new(program, "매틱");
            let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
            defaults.insert("값".to_string(), RuntimeValue::Fixed64(lhs));
            defaults.insert("다른값".to_string(), RuntimeValue::Fixed64(rhs));
            defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

            let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults)
            {
                Ok(_) => panic!("Fixed64 infix {operator} overflow must reject before publication"),
                Err(error) => error,
            };
            assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
        }
    }

    #[test]
    fn fixed64_infix_division_overflow_is_not_misreported_as_zero_division() {
        let script = r#"
매틱:움직씨 = {
    결과 <- 값 / 작은값.
}
"#;
        let program = DdnProgram::from_source(script, "fixed64_infix_div_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::MAX));
        defaults.insert(
            "작은값".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_raw_i64(1)),
        );
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));

        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults) {
            Ok(_) => panic!("nonzero tiny divisor must report overflow before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn fixed64_unit_suffix_overflow_rejects_before_publication() {
        let script = r#"
매틱:움직씨 = {
    결과 <- 2147483647@C == 0@K.
}
"#;
        let program = DdnProgram::from_source(script, "fixed64_unit_suffix_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Bool(false));

        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &defaults) {
            Ok(_) => panic!("unit suffix overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(
            error.diagnostic_parts().0,
            "E_RUNTIME_ARITH_OVERFLOW",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn fixed64_formula_pow_overflow_is_typed_and_does_not_saturate() {
        let error = eval_formula_pow(
            UnitValue {
                value: Fixed64::MAX,
                dim: UnitDim::NONE,
            },
            UnitValue {
                value: Fixed64::from_i64(2),
                dim: UnitDim::NONE,
            },
        )
        .expect_err("Fixed64 formula power overflow must reject");
        assert!(matches!(error, EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow)));
    }

    #[test]
    fn fixed64_formula_unary_neg_min_is_typed_overflow() {
        let expr = FormulaExpr::Unary {
            op: FormulaOp::Sub,
            expr: Box::new(FormulaExpr::Number(UnitValue {
                value: Fixed64::MIN,
                dim: UnitDim::NONE,
            })),
        };
        let error = eval_formula_expr(&expr, &BTreeMap::new())
            .expect_err("Fixed64 formula unary negation of minimum must reject");
        assert!(matches!(
            error,
            EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow)
        ));
    }

    #[test]
    fn fixed64_formula_nonfinite_host_results_must_not_publish_as_numbers() {
        assert_eq!(
            eval_formula_func(
                "sin",
                vec![UnitValue {
                    value: Fixed64::ZERO,
                    dim: UnitDim::NONE,
                }],
            )
            .expect("finite host result remains supported")
            .value,
            Fixed64::ZERO
        );

        for (name, value) in [
            ("asin", Fixed64::from_i64(2)),
            ("acos", Fixed64::from_i64(2)),
            ("acosh", Fixed64::ZERO),
            ("atanh", Fixed64::ONE),
        ] {
            let error = eval_formula_func(
                name,
                vec![UnitValue {
                    value,
                    dim: UnitDim::NONE,
                }],
            )
            .expect_err("non-finite host result must fail closed");
            assert!(matches!(
                error,
                EvalError::ArithmeticFault(ArithmeticFaultKind::Overflow)
            ));
        }
    }

    #[test]
    fn fixed64_finite_host_ingress_rounds_half_ulp_to_even() {
        let half_ulp = 0.5 / Fixed64::SCALE_F64;
        assert_eq!(
            fixed64_from_host_f64_checked(half_ulp),
            Ok(Fixed64::ZERO),
            "zero is the even neighbor at +0.5 raw"
        );
        assert_eq!(
            fixed64_from_host_f64_checked(3.0 * half_ulp),
            Ok(Fixed64::from_raw_i64(2)),
            "two is the even neighbor at +1.5 raw"
        );
        assert_eq!(
            fixed64_from_host_f64_checked(-half_ulp),
            Ok(Fixed64::ZERO),
            "zero is the even neighbor at -0.5 raw"
        );
        assert_eq!(
            fixed64_from_host_f64_checked(-3.0 * half_ulp),
            Ok(Fixed64::from_raw_i64(-2)),
            "negative ties retain the same even-neighbor rule"
        );
    }

    #[test]
    fn formula_constants_use_the_checked_host_fixed64_ingress() {
        let env = BTreeMap::new();
        let pi = eval_formula_expr(&FormulaExpr::Var("pi".to_string()), &env)
            .expect("pi constant must evaluate");
        let e = eval_formula_expr(&FormulaExpr::Var("e".to_string()), &env)
            .expect("e constant must evaluate");

        assert_eq!(pi.dim, UnitDim::NONE);
        assert_eq!(pi.value, Fixed64::from_raw_i64(13_493_037_705));
        assert_eq!(e.dim, UnitDim::NONE);
        assert_eq!(e.value, Fixed64::from_raw_i64(11_674_931_555));
    }

    #[test]
    fn formula_derivative_log_constants_use_checked_host_fixed64_ingress() {
        let FormulaExpr::Number(ln10) = num_ln10().expect("ln10 constant") else {
            panic!("ln10 must be a numeric formula expression");
        };
        let FormulaExpr::Number(ln2) = num_ln2().expect("ln2 constant") else {
            panic!("ln2 must be a numeric formula expression");
        };

        assert_eq!(ln10.dim, UnitDim::NONE);
        assert_eq!(ln10.value, Fixed64::from_raw_i64(9_889_527_671));
        assert_eq!(ln2.dim, UnitDim::NONE);
        assert_eq!(ln2.value, Fixed64::from_raw_i64(2_977_044_472));
    }

    #[test]
    fn numeric_calculus_v1_builtins_return_value_error_method() {
        let script = r#"
매틱:움직씨 = {
    미분묶음 <- (((#ascii) 수식{x^3}), "x", 1, 0.5) 미분.중앙차분.
    미분값 <- (미분묶음, 0) 차림.값.
    미분오차 <- (미분묶음, 1) 차림.값.
    미분방법 <- (미분묶음, 2) 차림.값.

    적분묶음 <- (((#ascii) 수식{x^2}), "x", 0, 1, 0.5) 적분.사다리꼴.
    적분값 <- (적분묶음, 0) 차림.값.
    적분오차 <- (적분묶음, 1) 차림.값.
    적분방법 <- (적분묶음, 2) 차림.값.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_calculus_v1.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미분값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("미분오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("미분방법".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("적분값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("적분오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("적분방법".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "미분값"),
            Fixed64::from_f64_lossy(3.0)
        );
        assert_eq!(
            extract_fixed(&output.resources, "미분오차"),
            Fixed64::from_f64_lossy(0.25)
        );
        match output.resources.get("미분방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "중앙차분"),
            other => panic!("미분방법 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "적분값"),
            Fixed64::from_raw_i64(1_431_655_765)
        );
        assert_eq!(
            extract_fixed(&output.resources, "적분오차"),
            Fixed64::from_raw_i64(44_739_243)
        );
        match output.resources.get("적분방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "사다리꼴"),
            other => panic!("적분방법 must be string, got {:?}", other),
        }
        assert_eq!(output.work_cost.counters.solver_evaluations, 12);
        assert_eq!(output.work_cost.counters.call_depth_peak, 2);
    }

    #[test]
    fn numeric_calculus_v1_rejects_zero_step() {
        let script = r#"
매틱:움직씨 = {
    d <- (((#ascii) 수식{x^2}), "x", 1, 0) 미분.중앙차분.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_calculus_v1_bad_step.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_CALC_NUMERIC_BAD_STEP"),
            "expected numeric step guard, got {err}"
        );
    }

    #[test]
    fn numeric_central_difference_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    d <- (((#ascii) 수식{x}), "x", 2147483647.99999999976716935634613037109375, 1) 미분.중앙차분.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_central_difference_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("central-difference overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn numeric_central_difference_checked_path_preserves_the_supported_result() {
        let script = r#"
매틱:움직씨 = {
    d <- (((#ascii) 수식{x^3}), "x", 1, 0.5) 미분.중앙차분.
    값 <- (d, 0) 차림.값.
    오차 <- (d, 1) 차림.값.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_central_difference_checked.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("checked central difference must preserve the supported result");
        assert_eq!(extract_fixed(&output.resources, "값"), Fixed64::from_i64(3));
        assert_eq!(
            extract_fixed(&output.resources, "오차"),
            Fixed64::from_f64_lossy(0.25)
        );
    }

    #[test]
    fn numeric_trapezoid_integral_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    i <- (((#ascii) 수식{x - x}), "x", -2147483647, 2147483647, 2147483647) 적분.사다리꼴.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_trapezoid_integral_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("trapezoid interval overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(
            error.diagnostic_parts().0,
            "E_RUNTIME_ARITH_OVERFLOW",
            "unexpected diagnostic: {error:?}"
        );
    }

    #[test]
    fn numeric_trapezoid_integral_checked_path_preserves_the_supported_result() {
        let script = r#"
매틱:움직씨 = {
    i <- (((#ascii) 수식{x^2}), "x", 0, 1, 0.5) 적분.사다리꼴.
    값 <- (i, 0) 차림.값.
    오차 <- (i, 1) 차림.값.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_trapezoid_integral_checked.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("checked trapezoid path must preserve the supported result");
        assert_eq!(
            extract_fixed(&output.resources, "값"),
            Fixed64::from_raw_i64(1_431_655_765)
        );
        assert_eq!(
            extract_fixed(&output.resources, "오차"),
            Fixed64::from_raw_i64(44_739_243)
        );
    }

    #[test]
    fn numeric_root_finding_v1_bisection_returns_root_residual_iteration_method() {
        let script = r#"
매틱:움직씨 = {
    r <- (((#ascii) 수식{x - 2}), "x", 0, 4, 8) 수치해.이분법.
    근 <- (r, 0) 차림.값.
    잔차 <- (r, 1) 차림.값.
    반복값 <- (r, 2) 차림.값.
    방법 <- (r, 3) 차림.값.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_root_finding_v1.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("근".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("잔차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("반복값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("방법".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "근"), Fixed64::from_i64(2));
        assert_eq!(extract_fixed(&output.resources, "잔차"), Fixed64::ZERO);
        assert_eq!(
            extract_fixed(&output.resources, "반복값"),
            Fixed64::from_i64(1)
        );
        match output.resources.get("방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "이분법"),
            other => panic!("방법 must be string, got {:?}", other),
        }
        assert_eq!(output.work_cost.counters.solver_evaluations, 3);
        assert_eq!(output.work_cost.counters.call_depth_peak, 2);
    }

    #[test]
    fn numeric_root_finding_v1_rejects_unbracketed_root() {
        let script = r#"
매틱:움직씨 = {
    r <- (((#ascii) 수식{x^2 + 1}), "x", -1, 1, 8) 수치해.이분법.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_root_finding_v1_bad.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_CALC_NUMERIC_BRACKET_SIGN"),
            "expected bracket sign guard, got {err}"
        );
        assert_eq!(runner.work_cost_receipt().counters.solver_evaluations, 2);
        assert_eq!(runner.work_cost_receipt().counters.call_depth_current, 0);
        assert_eq!(runner.work_cost_receipt().counters.call_depth_peak, 2);
    }

    #[test]
    fn numeric_bisection_midpoint_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    r <- (((#ascii) 수식{x - 2000000000}), "x", 1500000000, 2147483647, 1) 수치해.이분법.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_bisection_midpoint_overflow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("bisection midpoint overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn range_step_overflow_is_typed_and_publishes_nothing() {
        let script = r#"
매틱:움직씨 = {
    r <- (2147483646, 2147483647, 2) 범위.
}
"#;
        let program = DdnProgram::from_source(script, "range_step_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("range step overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn range_endpoint_at_fixed64_max_stops_without_saturation_loop() {
        let script = r#"
매틱:움직씨 = {
    r <- (2147483647, 2147483647, 1) 범위.
}
"#;
        let program = DdnProgram::from_source(script, "range_endpoint_max.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("r".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("endpoint range must complete");
        assert_eq!(
            output.resources.get("r"),
            Some(&RuntimeValue::List(vec![RuntimeValue::Fixed64(Fixed64::from_i64(2_147_483_647))]))
        );
    }

    #[test]
    fn polynomial_solve_minimum_v1_reuses_exact_quadratic_relation_solver() {
        let script = r#"
매틱:움직씨 = {
    f <- (#ascii) 수식{x^2 - 4*x + 4}.
    결과 <- (f, "x") 다항식.풀기.
    미지수 <- 결과.미지수.
    값 <- 결과.값.
}
"#;
        let program =
            DdnProgram::from_source(script, "polynomial_solve_minimum_v1.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미지수".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("미지수") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "x"),
            other => panic!("미지수 must be string, got {:?}", other),
        }
        let Some(RuntimeValue::Pack(value_pack)) = output.resources.get("값") else {
            panic!("값 must be exact numeric pack");
        };
        assert_eq!(
            value_pack.get("__수타입"),
            Some(&RuntimeValue::String("큰바른수".to_string()))
        );
        assert_eq!(
            value_pack.get("값"),
            Some(&RuntimeValue::String("2".to_string()))
        );
    }

    #[test]
    fn polynomial_solve_minimum_v1_preserves_non_unique_boundary() {
        let script = r#"
매틱:움직씨 = {
    f <- (#ascii) 수식{x^2 - 5*x + 6}.
    결과 <- (f, "x") 다항식.풀기.
    사유 <- 결과.사유.
}
"#;
        let program = DdnProgram::from_source(script, "polynomial_solve_minimum_v1_non_unique.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("사유".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("사유") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "non_unique"),
            other => panic!("사유 must be string, got {:?}", other),
        }
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_returns_bounded_interval() {
        let conditions = Value::List(vec![
            linear_inequality_test_condition("2*x + 1", "이하", 9),
            linear_inequality_test_condition("x - 1", "이상", 2),
        ]);
        let work_cost = Rc::new(RefCell::new(WorkCostMeter::default()));
        let solved =
            eval_linear_inequality_solve(&[conditions, Value::String("x".to_string())], &work_cost)
                .expect("solve");
        let Value::Pack(output) = &solved else {
            panic!("solution must be pack");
        };
        match output.get("상태") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "구간"),
            other => panic!("상태 must be string, got {:?}", other),
        }
        assert_eq!(
            output.get("하한"),
            Some(&unit_value_to_value(UnitValue {
                value: Fixed64::from_i64(3),
                dim: UnitDim::NONE,
            }))
        );
        assert_eq!(
            output.get("상한"),
            Some(&unit_value_to_value(UnitValue {
                value: Fixed64::from_i64(4),
                dim: UnitDim::NONE,
            }))
        );
        assert_eq!(output.get("하한포함"), Some(&RuntimeValue::Bool(true)));
        assert_eq!(output.get("상한포함"), Some(&RuntimeValue::Bool(true)));
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_detects_empty_interval() {
        let conditions = Value::List(vec![
            linear_inequality_test_condition("x", "미만", 1),
            linear_inequality_test_condition("x", "초과", 1),
        ]);
        let work_cost = Rc::new(RefCell::new(WorkCostMeter::default()));
        let solved =
            eval_linear_inequality_solve(&[conditions, Value::String("x".to_string())], &work_cost)
                .expect("solve");
        let Value::Pack(output) = &solved else {
            panic!("solution must be pack");
        };
        match output.get("상태") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "공집합"),
            other => panic!("상태 must be string, got {:?}", other),
        }
    }

    #[test]
    fn linear_inequality_rhs_overflow_is_typed_and_publishes_nothing() {
        let conditions = Value::List(vec![linear_inequality_test_condition(
            "x - 2147483647",
            "이하",
            2_147_483_647,
        )]);
        let work_cost = Rc::new(RefCell::new(WorkCostMeter::default()));
        let error = eval_linear_inequality_solve(
            &[conditions, Value::String("x".to_string())],
            &work_cost,
        )
        .expect_err("unrepresentable RHS must reject before solution publication");
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn contract_pre_abort_restores_state_before_else_body() {
        let script = r#"
매틱:움직씨 = {
    x <- 5.
    { x > 10 }인것 바탕으로(물림) 아니면 {
        x <- x + 1.
    }
    y <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "contract_pre_abort.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("x".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "x"), Fixed64::from_i64(5));
        assert!(!output.resources.contains_key("y"));
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("물림"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("pre"));
    }

    #[test]
    fn contract_post_abort_restores_state_before_repair_body() {
        let script = r#"
매틱:움직씨 = {
    y <- 5.
    { y < 5 }인것 다짐하고(물림) 아니면 {
        y <- y + 1.
    }
    z <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "contract_post_abort.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("z".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "y"), Fixed64::from_i64(5));
        assert!(!output.resources.contains_key("z"));
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("물림"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("post"));
    }

    #[test]
    fn contract_alert_keeps_else_body_state_changes() {
        let script = r#"
매틱:움직씨 = {
    z <- 5.
    { z > 10 }인것 바탕으로(알림) 아니면 {
        z <- z + 1.
    }
    확인 <- z.
}
"#;
        let program = DdnProgram::from_source(script, "contract_alert.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("z".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("확인".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "z"), Fixed64::from_i64(6));
        assert_eq!(
            extract_fixed(&output.resources, "확인"),
            Fixed64::from_i64(6)
        );
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("알림"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("pre"));
    }

    #[test]
    fn tailed_call_alias_registers_myeonseo_variant() {
        let script = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

매틱:움직씨 = {
    합 <- (3을, 1에) 더하면서.
}
"#;
        let program = DdnProgram::from_source(script, "tail_alias_myeonseo.ddn").expect("parse");
        assert!(program.functions.contains_key("더하면서"));

        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("합".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(extract_fixed(&output.resources, "합"), Fixed64::from_i64(4));
    }

    #[test]
    fn state_machine_builtins_progress_and_store_value() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_안전 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
        바뀔때마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
    확인 <- (기계, 다음) 지금상태.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("현재".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("다음".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("확인".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("기계") {
            Some(RuntimeValue::StateMachine(machine)) => {
                assert_eq!(machine.initial, "빨강");
                assert_eq!(machine.transitions.len(), 3);
            }
            other => panic!("기계 must be state machine, got {:?}", other),
        }
        match output.resources.get("현재") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "빨강"),
            other => panic!("현재 must be string, got {:?}", other),
        }
        match output.resources.get("다음") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "초록"),
            other => panic!("다음 must be string, got {:?}", other),
        }
        match output.resources.get("확인") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "초록"),
            other => panic!("확인 must be string, got {:?}", other),
        }
        let events = state_transition_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "STATE_TRANSITION");
        assert_eq!(events[0].sub_reason.as_deref(), Some("NEXT"));
        assert_eq!(
            events[0].message.as_deref(),
            Some("상태 전이: 빨강 -> 초록")
        );
        assert_eq!(output.state_transitions.len(), 1);
        assert_eq!(output.state_transitions[0].madi, 0);
        assert_eq!(output.state_transitions[0].from, "빨강");
        assert_eq!(output.state_transitions[0].to, "초록");
        assert_eq!(
            output.state_transitions[0].message,
            "상태 전이: 빨강 -> 초록"
        );
    }

    #[test]
    fn state_machine_transition_check_failure_aborts_next_state() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_실패 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        바뀔때마다 전이_실패 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine_fail.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("transition check must fail"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_CHECK_FAILED: 전이_실패"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_transition_check_requires_resolved_assertion() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        바뀔때마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_unresolved.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("missing transition assertion must fail"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_CHECK_UNRESOLVED: 전이_안전"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_guard_selects_only_passing_transition_and_runs_action() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
(현재상태:글, 다음상태:글) 기록:움직씨 = {
    횟수 <- 횟수 + 1.
    마지막 <- 다음상태.
}

매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 거짓_거름 하고 기록.
        빨강 에서 파랑 으로 걸러서 참_거름 하고 기록.
        바뀔때마다 참_거름 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_action.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("현재".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("다음".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("마지막".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("다음") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "파랑"),
            other => panic!("다음 must be string, got {:?}", other),
        }
        match output.resources.get("마지막") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "파랑"),
            other => panic!("마지막 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "횟수"),
            Fixed64::from_i64(1)
        );
        assert_eq!(output.state_transitions.len(), 1);
        assert_eq!(output.state_transitions[0].from, "빨강");
        assert_eq!(output.state_transitions[0].to, "파랑");
        assert_eq!(
            output.state_transitions[0].guard_name.as_deref(),
            Some("참_거름")
        );
        assert_eq!(
            output.state_transitions[0].action_name.as_deref(),
            Some("기록")
        );
    }

    #[test]
    fn state_machine_guard_rejects_multiple_passing_transitions_without_action_publish() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
(현재상태:글, 다음상태:글) 기록:움직씨 = {
    횟수 <- 횟수 + 1.
    마지막 <- 다음상태.
}

매틱:움직씨 = {
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 참_거름 하고 기록.
        빨강 에서 파랑 으로 걸러서 참_거름 하고 기록.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_ambiguous.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("현재".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("다음".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("마지막".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("multiple passing transitions must fail closed"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("E_STATE_TRANSITION_GUARD_REJECTED: 빨강"),
            "unexpected error: {err}"
        );
        assert_eq!(
            defaults.get("횟수"),
            Some(&RuntimeValue::Fixed64(Fixed64::ZERO))
        );
        assert_eq!(
            defaults.get("마지막"),
            Some(&RuntimeValue::String(String::new()))
        );
    }

    #[test]
    fn state_machine_guard_evaluates_whole_set_before_selecting_transition() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 참_거름.
        빨강 에서 파랑 으로 걸러서 없는_거름.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_whole_set.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("unresolved guard anywhere in the eligible set must abort selection"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("E_STATE_TRANSITION_GUARD_UNRESOLVED: 없는_거름"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn state_machine_guard_fault_aborts_whole_candidate_set_without_selection() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    깨진_거름 <- 세움{
        { 없는값 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 깨진_거름.
        빨강 에서 파랑 으로 걸러서 참_거름.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_fault.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("a guard evaluation fault must abort the complete candidate set"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("E_STATE_TRANSITION_GUARD_REJECTED: 깨진_거름"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn state_machine_guard_rejected_when_all_candidates_fail() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 거짓_거름.
        빨강 에서 파랑 으로 걸러서 거짓_거름.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_rejected.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("guard reject expected"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_GUARD_REJECTED: 빨강"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_is_age1_feature_gated() {
        let script = r#"
매틱:움직씨 = {
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
    }.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "state_machine_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate, got {err}"
        );
        assert!(
            err.contains("state_machine_literal"),
            "expected state machine feature, got {err}"
        );
    }

    #[test]
    fn assertion_check_returns_true_and_stores_value() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_ok.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("검사") {
            Some(RuntimeValue::Assertion(assertion)) => {
                assert!(assertion.canon.starts_with("세움{"));
            }
            other => panic!("검사 must be assertion, got {:?}", other),
        }
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn assertion_check_failure_emits_diag_and_keeps_state_clean() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    x <- 1.
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=0)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_fail.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("x".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("결과".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(extract_fixed(&output.resources, "x"), Fixed64::from_i64(1));
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(!*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
        let events = assertion_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "ASSERTION_FAILED");
    }

    #[test]
    fn assertion_is_age1_feature_gated() {
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "assertion_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate, got {err}"
        );
        assert!(
            err.contains("assertion_literal") || err.contains("assertion_api"),
            "expected assertion feature, got {err}"
        );
    }

    #[test]
    fn seumssi_v1b_surface_alias_runs_as_canonical_seum() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움씨{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "seumssi_v1b.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        match output.resources.get("검사") {
            Some(RuntimeValue::Assertion(assertion)) => {
                assert!(assertion.canon.starts_with("세움{"));
                assert!(!assertion.canon.starts_with("세움씨{"));
            }
            other => panic!("검사 must be assertion, got {:?}", other),
        }
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn regex_builtins_run_deterministically() {
        let script = r#"
매틱:움직씨 = {
    맞음 <- ("ab12", 정규식{"^[A-Z]{2}[0-9]+$", "i"}) 정규맞추기.
    첫매치 <- ("x12y34", 정규식{"[0-9]+"}) 정규찾기.
    캡처 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}) 정규캡처하기.
    이름캡처 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}) 정규이름캡처하기.
    이름선택캡처 <- ("ab", 정규식{"(?P<word>[a-z]+)(?:-(?P<num>[0-9]+))?", "i"}) 정규이름캡처하기.
    이름캡처없음 <- ("12", 정규식{"(?P<word>[a-z]+)", "i"}) 정규이름캡처하기.
    바꿈 <- ("a1b2", 정규식{"[0-9]+"}, "_") 정규바꾸기.
    재바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$2:$1") 정규바꾸기.
    이름재바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${num}:${word}") 정규바꾸기.
    이름짧은재바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "$num:$word") 정규바꾸기.
    조각 <- ("a1b22c", 정규식{"[0-9]+"}) 정규나누기.
    캡처조각 <- ("ab-12-cd", 정규식{"(-)([0-9]+)(-)"}) 정규나누기.
    줄매치 <- ("x\nAB\ny", 정규식{"^ab$", "mi"}) 정규찾기.
    점맞음 <- ("A\nb", 정규식{"a.b", "si"}) 정규맞추기.
    조각수 <- (조각) 길이.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("맞음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("첫매치".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("캡처".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("이름캡처".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert(
            "이름선택캡처".to_string(),
            RuntimeValue::Map(BTreeMap::new()),
        );
        defaults.insert(
            "이름캡처없음".to_string(),
            RuntimeValue::Map(BTreeMap::new()),
        );
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("재바꿈".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "이름재바꿈".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert(
            "이름짧은재바꿈".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert("캡처조각".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("줄매치".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("점맞음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("조각수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("맞음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("맞음 must be bool, got {:?}", other),
        }
        match output.resources.get("첫매치") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12"),
            other => panic!("첫매치 must be string, got {:?}", other),
        }
        match output.resources.get("캡처") {
            Some(RuntimeValue::List(items)) => {
                let strings: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        RuntimeValue::String(value) => value.clone(),
                        other => panic!("캡처 item must be string, got {:?}", other),
                    })
                    .collect();
                assert_eq!(strings, vec!["ab-12", "ab", "12"]);
            }
            other => panic!("캡처 must be list, got {:?}", other),
        }
        match output.resources.get("이름캡처") {
            Some(RuntimeValue::Map(entries)) => {
                assert_eq!(entries.len(), 2);
                let word = entries.get("\"word\"").expect("word capture");
                let num = entries.get("\"num\"").expect("num capture");
                assert_eq!(value_canon(&word.value), "\"ab\"");
                assert_eq!(value_canon(&num.value), "\"12\"");
            }
            other => panic!("이름캡처 must be map, got {:?}", other),
        }
        match output.resources.get("이름선택캡처") {
            Some(RuntimeValue::Map(entries)) => {
                assert_eq!(entries.len(), 2);
                let word = entries.get("\"word\"").expect("word optional capture");
                let num = entries.get("\"num\"").expect("num optional capture");
                assert_eq!(value_canon(&word.value), "\"ab\"");
                assert_eq!(value_canon(&num.value), "\"\"");
            }
            other => panic!("이름선택캡처 must be map, got {:?}", other),
        }
        match output.resources.get("이름캡처없음") {
            Some(RuntimeValue::Map(entries)) => assert!(entries.is_empty()),
            other => panic!("이름캡처없음 must be map, got {:?}", other),
        }
        match output.resources.get("바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "a_b_"),
            other => panic!("바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("이름재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("이름재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("이름짧은재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("이름짧은재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("캡처조각") {
            Some(RuntimeValue::List(items)) => {
                let strings: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        RuntimeValue::String(value) => value.clone(),
                        other => panic!("캡처조각 item must be string, got {:?}", other),
                    })
                    .collect();
                assert_eq!(strings, vec!["ab".to_string(), "cd".to_string()]);
            }
            other => panic!("캡처조각 must be list, got {:?}", other),
        }
        match output.resources.get("줄매치") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "AB"),
            other => panic!("줄매치 must be string, got {:?}", other),
        }
        match output.resources.get("점맞음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("점맞음 must be bool, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "조각수"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn stream_stdlib_push_and_read_order() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (3) 흐름.만들기.
    흐름 <- (흐름, "a") 흐름.밀어넣기.
    흐름 <- (흐름, "b") 흐름.밀어넣기.
    흐름 <- (흐름, "c") 흐름.밀어넣기.
    흐름 <- (흐름, "d") 흐름.밀어넣기.
    값들 <- (흐름) 흐름.차림.
    최근 <- (흐름) 흐름.최근값.
    길이 <- (흐름) 흐름.길이.
    용량 <- (흐름) 흐름.용량.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("용량".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("값들") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_string(value))
                    .collect::<Vec<_>>();
                assert_eq!(
                    rendered,
                    vec!["b".to_string(), "c".to_string(), "d".to_string()]
                );
            }
            other => panic!("값들 must be list, got {:?}", other),
        }
        match output.resources.get("최근") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "d"),
            other => panic!("최근 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "길이"),
            Fixed64::from_i64(3)
        );
        assert_eq!(
            extract_fixed(&output.resources, "용량"),
            Fixed64::from_i64(3)
        );
        match output.resources.get("흐름") {
            Some(RuntimeValue::Map(entries)) => {
                let schema = entries.get("\"__schema\"").expect("__schema");
                assert_eq!(value_canon(&schema.value), "\"ddn.stream.v1\"");
            }
            other => panic!("흐름 must be map, got {:?}", other),
        }
    }

    #[test]
    fn stream_stdlib_aliases_are_supported() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (2) 흐름만들기.
    흐름 <- (흐름, 1) 흐름추가.
    흐름 <- (흐름, 2) 흐름넣기.
    흐름 <- (흐름, 3) 흐름추가.
    값들 <- (흐름) 흐름값들.
    최근 <- (흐름) 흐름최근.
    최근하나 <- (흐름, 1) 흐름잘라보기.
    최근둘 <- (흐름, 2) 흐름최근N.
    흐름 <- (흐름) 흐름비우기.
    비운길이 <- (흐름) 흐름길이.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib_alias.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("최근하나".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근둘".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("비운길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("값들") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![2, 3]);
            }
            other => panic!("값들 must be list, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "최근"),
            Fixed64::from_i64(3)
        );
        match output.resources.get("최근하나") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![3]);
            }
            other => panic!("최근하나 must be list, got {:?}", other),
        }
        match output.resources.get("최근둘") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![2, 3]);
            }
            other => panic!("최근둘 must be list, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "비운길이"), Fixed64::ZERO);
    }

    #[test]
    fn stream_stdlib_clear_and_slice_work() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (3) 흐름.만들기.
    흐름 <- (흐름, 1) 흐름.밀어넣기.
    흐름 <- (흐름, 2) 흐름.밀어넣기.
    흐름 <- (흐름, 3) 흐름.밀어넣기.
    흐름 <- (흐름, 4) 흐름.밀어넣기.
    최근둘 <- (흐름, 2) 흐름.잘라보기.
    흐름 <- (흐름) 흐름.비우기.
    비운길이 <- (흐름) 흐름.길이.
    비운값들 <- (흐름) 흐름.차림.
}
"#;
        let program =
            DdnProgram::from_source(script, "stream_stdlib_clear_slice.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("최근둘".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("비운길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("비운값들".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("최근둘") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![3, 4]);
            }
            other => panic!("최근둘 must be list, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "비운길이"), Fixed64::ZERO);
        match output.resources.get("비운값들") {
            Some(RuntimeValue::List(items)) => assert!(items.is_empty()),
            other => panic!("비운값들 must be list, got {:?}", other),
        }
    }

    #[test]
    fn stream_stdlib_reports_consistent_stream_value_label() {
        let script = r#"
매틱:움직씨 = {
    길이 <- (흐름) 흐름.길이.
}
"#;
        let program =
            DdnProgram::from_source(script, "stream_stdlib_label_error.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("흐름(흐름.만들기 결과) 인자가 필요합니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_invalid_returns_dedicated_error() {
        let script = r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${missing}") 정규바꾸기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program =
            DdnProgram::from_source(script, "regex_replacement_invalid.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_numeric_backref_is_greedy_and_invalid_when_missing() {
        let script = r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$10") 정규바꾸기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex_replacement_numeric_invalid.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_invalid_subcases_return_dedicated_error() {
        let cases = [
            (
                "regex_replacement_empty_ref_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${}") 정규바꾸기.
}
"#,
            ),
            (
                "regex_replacement_dangling_dollar_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "$") 정규바꾸기.
}
"#,
            ),
            (
                "regex_replacement_numeric_overflow_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$999999999999999999999999") 정규바꾸기.
}
"#,
            ),
        ];
        for (file_name, script) in cases {
            let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
            let program = DdnProgram::from_source(script, file_name).expect("parse");
            let mut runner = DdnRunner::new(program, "매틱");
            let world = NuriWorld::new();
            let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
            defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
            let err = match runner.run_update(&world, &empty_input(), &defaults) {
                Ok(_) => panic!("must fail: {file_name}"),
                Err(err) => err,
            };
            assert!(
                err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
                "unexpected error for {file_name}: {err}"
            );
        }
    }

    #[test]
    fn temperature_literals_compare_after_kelvin_normalization() {
        let script = r#"
매틱:움직씨 = {
    같음 <- 25@C == 77@F.
}
"#;
        let program = DdnProgram::from_source(script, "temperature_compare.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("같음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("같음 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn temperature_literals_subtract_to_kelvin_difference() {
        let script = r#"
매틱:움직씨 = {
    같음 <- (30@C - 20@C) == 10@K.
}
"#;
        let program = DdnProgram::from_source(script, "temperature_subtract.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("같음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("같음 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn std_grid_closure_runtime_pathfind_and_physics_smoke() {
        let script = r##"
매틱:움직씨 = {
    길 <- ".".
    벽 <- "#".
    격0 <- (4, 3, 길) 격자.만들기.
    격1 <- (격0, 1, 0, 벽) 격자.바꾼값.
    격2 <- (격1, 1, 1, 벽) 격자.바꾼값.
    격3 <- (격2, 3, 1, 벽) 격자.바꾼값.
    경로 <- (격3, 0, 0, 3, 2, (벽) 차림) 격자.길찾기.
    경로길이 <- (경로) 길이.
    다음위치 <- (10, 2, 3) 물리1d.위치갱신.
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "경로길이".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        defaults.insert(
            "다음위치".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "경로길이"),
            Fixed64::from_i64(6)
        );
        assert_eq!(
            extract_fixed(&output.resources, "다음위치"),
            Fixed64::from_i64(16)
        );
    }

    #[test]
    fn physics_position_update_overflow_is_typed_and_publishes_nothing() {
        let script = r##"
매틱:움직씨 = {
    다음위치 <- (2147483647.99999999976716935634613037109375, 1, 1) 물리1d.위치갱신.
}
"##;
        let program = DdnProgram::from_source(script, "physics_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "다음위치".to_string(),
            RuntimeValue::Fixed64(Fixed64::ZERO),
        );

        let error = match runner.run_update_diagnostic(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("overflow must reject before publication"),
            Err(error) => error,
        };
        let (code, _message, _span) = error.diagnostic_parts();
        assert_eq!(code, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn physics_velocity_update_overflow_is_typed_and_publishes_nothing() {
        let script = r##"
매틱:움직씨 = {
    다음속도 <- (2147483647.99999999976716935634613037109375, 1, 1) 물리1d.속도갱신.
}
"##;
        let program = DdnProgram::from_source(script, "physics_velocity_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "다음속도".to_string(),
            RuntimeValue::Fixed64(Fixed64::ZERO),
        );

        let error = match runner.run_update_diagnostic(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("overflow must reject before publication"),
            Err(error) => error,
        };
        let (code, _message, _span) = error.diagnostic_parts();
        assert_eq!(code, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn physics_elastic_collision_uses_checked_fixed64_and_rejects_overflow() {
        let normal_script = r##"
매틱:움직씨 = {
    충돌 <- (1, 2, 1, 0) 물리1d.탄성충돌1d.
}
"##;
        let normal_program =
            DdnProgram::from_source(normal_script, "physics_collision.ddn").expect("parse");
        let mut normal_runner = DdnRunner::new(normal_program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("충돌".to_string(), RuntimeValue::List(Vec::new()));
        let output = normal_runner
            .run_update(&NuriWorld::new(), &empty_input(), &defaults)
            .expect("normal collision");
        let collision = match output.resources.get("충돌") {
            Some(RuntimeValue::List(items)) => items,
            other => panic!("collision must be a pair, got {other:?}"),
        };
        assert_eq!(collision.len(), 2);
        assert_eq!(collision[0], RuntimeValue::Fixed64(Fixed64::ZERO));
        assert_eq!(collision[1], RuntimeValue::Fixed64(Fixed64::from_i64(2)));

        let overflow_script = r##"
매틱:움직씨 = {
    충돌 <- (1073741824, 1, 0, 0) 물리1d.탄성충돌1d.
}
"##;
        let overflow_program =
            DdnProgram::from_source(overflow_script, "physics_collision_overflow.ddn")
                .expect("parse");
        let mut overflow_runner = DdnRunner::new(overflow_program, "매틱");
        let error = match overflow_runner.run_update_diagnostic(
            &NuriWorld::new(),
            &empty_input(),
            &defaults,
        ) {
            Ok(_) => panic!("overflow must reject before publication"),
            Err(error) => error,
        };
        assert_eq!(error.diagnostic_parts().0, "E_RUNTIME_ARITH_OVERFLOW");
    }

    #[test]
    fn std_block_piece_geometry_moves_and_rotates() {
        let script = r#"
매틱:움직씨 = {
    조각 <- (((0, 0) 차림, (1, 0) 차림, (0, 1) 차림) 차림) 블록조각.만들기.
    원본 <- (조각) 블록조각.칸목록.
    이동 <- ((조각, 2, 1) 블록조각.이동) 블록조각.칸목록.
    오른쪽 <- ((조각, "오른쪽") 블록조각.회전) 블록조각.칸목록.
    왼쪽 <- ((조각, "왼쪽") 블록조각.회전) 블록조각.칸목록.
    뒤집기 <- ((조각, "뒤집기") 블록조각.회전) 블록조각.칸목록.
}
"#;
        let program =
            DdnProgram::from_source(script, "std_block_piece_geometry.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["원본", "이동", "오른쪽", "왼쪽", "뒤집기"] {
            defaults.insert(key.to_string(), RuntimeValue::List(Vec::new()));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("원본").map(value_to_string),
            Some("차림[차림[0, 0], 차림[1, 0], 차림[0, 1]]".to_string())
        );
        assert_eq!(
            output.resources.get("이동").map(value_to_string),
            Some("차림[차림[2, 1], 차림[3, 1], 차림[2, 2]]".to_string())
        );
        assert_eq!(
            output.resources.get("오른쪽").map(value_to_string),
            Some("차림[차림[-1, 0], 차림[0, 0], 차림[0, 1]]".to_string())
        );
        assert_eq!(
            output.resources.get("왼쪽").map(value_to_string),
            Some("차림[차림[0, -1], 차림[0, 0], 차림[1, 0]]".to_string())
        );
        assert_eq!(
            output.resources.get("뒤집기").map(value_to_string),
            Some("차림[차림[0, -1], 차림[-1, 0], 차림[0, 0]]".to_string())
        );
    }

    #[test]
    fn std_block_piece_grid_bridge_collides_and_locks() {
        let script = r##"
매틱:움직씨 = {
    빈칸 <- ".".
    벽 <- "#".
    격0 <- (4, 3, 빈칸) 격자.만들기.
    격1 <- (격0, 3, 1, 벽) 격자.바꾼값.
    조각 <- (((0, 0) 차림, (1, 0) 차림, (0, 1) 차림) 차림) 블록조각.만들기.
    안쪽 <- (조각, 1, 1) 블록조각.이동.
    벽쪽 <- (조각, 2, 1) 블록조각.이동.
    바깥 <- (조각, -1, 0) 블록조각.이동.
    안쪽충돌 <- (안쪽, 격1, (벽) 차림) 블록조각.충돌?.
    벽충돌 <- (벽쪽, 격1, (벽) 차림) 블록조각.충돌?.
    밖충돌 <- (바깥, 격1, (벽) 차림) 블록조각.충돌?.
    고정격 <- (안쪽, 격1, "X") 블록조각.고정.
    고정값 <- (고정격, 1, 1) 격자.값.
    원래벽 <- (고정격, 3, 1) 격자.값.
}
"##;
        let program =
            DdnProgram::from_source(script, "std_block_piece_grid_bridge.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["안쪽충돌", "벽충돌", "밖충돌"] {
            defaults.insert(key.to_string(), RuntimeValue::Bool(false));
        }
        defaults.insert("고정값".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원래벽".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("안쪽충돌"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("벽충돌"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("밖충돌"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("고정값"),
            Some(&RuntimeValue::String("X".to_string()))
        );
        assert_eq!(
            output.resources.get("원래벽"),
            Some(&RuntimeValue::String("#".to_string()))
        );
    }

    #[test]
    fn std_random_bag_draws_refills_and_previews() {
        let script = r#"
매틱:움직씨 = {
    가방0 <- (0, ("I", "O", "T") 차림) 무작위가방.만들기.
    미리 <- (가방0, 5) 무작위가방.미리보기.
    뽑1 <- (가방0) 무작위가방.꺼내기.
    값1 <- 뽑1.값.
    가방1 <- 뽑1.가방.
    뽑2 <- (가방1) 무작위가방.꺼내기.
    값2 <- 뽑2.값.
    가방2 <- 뽑2.가방.
    뽑3 <- (가방2) 무작위가방.꺼내기.
    값3 <- 뽑3.값.
    가방3 <- 뽑3.가방.
    빈가 <- (가방3) 무작위가방.비었나.
    남은3 <- (가방3) 무작위가방.남은것.
    뽑4 <- (가방3) 무작위가방.꺼내기.
    값4 <- 뽑4.값.
    남은4 <- (뽑4.가방) 무작위가방.남은것.
}
"#;
        let program = DdnProgram::from_source(script, "std_random_bag.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미리".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("값1".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값2".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값3".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값4".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("빈가".to_string(), RuntimeValue::Bool(false));
        defaults.insert("남은3".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("남은4".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("미리").map(value_to_string),
            Some("차림[O, I, T, O, T]".to_string())
        );
        assert_eq!(
            output.resources.get("값1"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("값2"),
            Some(&RuntimeValue::String("I".to_string()))
        );
        assert_eq!(
            output.resources.get("값3"),
            Some(&RuntimeValue::String("T".to_string()))
        );
        assert_eq!(
            output.resources.get("빈가"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("남은3").map(value_to_string),
            Some("차림[]".to_string())
        );
        assert_eq!(
            output.resources.get("값4"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("남은4").map(value_to_string),
            Some("차림[I, T]".to_string())
        );
    }

    #[test]
    fn std_grid_game_state_lifecycle_pause_resume() {
        let script = r#"
매틱:움직씨 = {
    초기 <- () 격자게임상태.초기화.
    상태0 <- (초기) 격자게임상태.상태.
    틱0 <- (초기) 격자게임상태.틱.
    진행 <- (초기, "진행") 격자게임상태.바꾸기.
    진행인가 <- (진행, "진행") 격자게임상태.상태인가.
    멈춤 <- (진행) 격자게임상태.멈춤.
    멈춤상태 <- (멈춤) 격자게임상태.상태.
    재개 <- (멈춤) 격자게임상태.재개.
    재개상태 <- (재개) 격자게임상태.상태.
    재개틱 <- (재개) 격자게임상태.틱.
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_state.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("상태0".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "틱0".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        defaults.insert("진행인가".to_string(), RuntimeValue::Bool(false));
        defaults.insert("멈춤상태".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("재개상태".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "재개틱".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("상태0"),
            Some(&RuntimeValue::String("준비".to_string()))
        );
        assert_eq!(
            output.resources.get("틱0"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("진행인가"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("멈춤상태"),
            Some(&RuntimeValue::String("멈춤".to_string()))
        );
        assert_eq!(
            output.resources.get("재개상태"),
            Some(&RuntimeValue::String("진행".to_string()))
        );
        assert_eq!(
            output.resources.get("재개틱"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(3)))
        );
    }

    #[test]
    fn std_grid_game_playable_catalog_lines_falling_score_and_tick() {
        let script = r#"
매틱:움직씨 = {
    이름들 <- () 테트로미노.이름목록.
    아이 <- ("I") 테트로미노.만들기.
    아이칸 <- (아이) 블록조각.칸목록.
    빈 <- ".".
    격0 <- (4, 3, 빈) 격자.만들기.
    격1 <- (격0, 1, 1, "P") 격자.바꾼값.
    격2 <- (격1, 0, 2, "X") 격자.바꾼값.
    격3 <- (격2, 1, 2, "X") 격자.바꾼값.
    격4 <- (격3, 2, 2, "X") 격자.바꾼값.
    격5 <- (격4, 3, 2, "X") 격자.바꾼값.
    찬줄 <- (격5, 빈) 격자줄.찬줄목록.
    지움 <- (격5, 빈) 격자줄.지우기.
    지운수 <- 지움.지운줄수.
    격후 <- 지움.격자.
    아래중 <- (격후, 1, 2) 격자.값.
    오 <- ("O") 테트로미노.만들기.
    낙0 <- (오, 2, 3) 낙하조각.만들기.
    배치0 <- (낙0) 낙하조각.배치.
    점0 <- () 격자게임점수.초기화.
    점1 <- (점0, 4) 격자게임점수.더하기.
    점2 <- (점1, 4) 격자게임점수.더하기.
    점3 <- (점2, 2) 격자게임점수.더하기.
    점4 <- (점3, 1) 격자게임점수.더하기.
    점수4 <- (점4) 격자게임점수.점수.
    줄4 <- (점4) 격자게임점수.줄수.
    레벨4 <- (점4) 격자게임점수.레벨.
    격 <- (4, 4, 빈) 격자.만들기.
    가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
    상태0 <- () 격자게임상태.초기화.
    진행상태 <- (상태0, "진행") 격자게임상태.바꾸기.
    낙 <- (오, 1, 0) 낙하조각.만들기.
    세션 <- (격, 가방, 진행상태, 점0, 낙) 격자게임세션.만들기.
    조작맵 <- () 입력사상.만들기.
    틱 <- (세션, 조작맵) 격자게임.한틱.
    고정 <- 틱.고정됐나.
    다음세션 <- 틱.세션.
    다음낙 <- (다음세션) 격자게임세션.낙하조각.
    위치 <- (다음낙) 낙하조각.위치.
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_playable.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["이름들", "아이칸", "찬줄", "배치0", "위치"] {
            defaults.insert(key.to_string(), RuntimeValue::List(Vec::new()));
        }
        for key in ["지운수", "점수4", "줄4", "레벨4"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        defaults.insert("아래중".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("고정".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("이름들").map(value_to_string),
            Some("차림[I, O, T, S, Z, J, L]".to_string())
        );
        assert_eq!(
            output.resources.get("아이칸").map(value_to_string),
            Some("차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[2, 0]]".to_string())
        );
        assert_eq!(
            output.resources.get("찬줄").map(value_to_string),
            Some("차림[2]".to_string())
        );
        assert_eq!(
            output.resources.get("지운수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("아래중"),
            Some(&RuntimeValue::String("P".to_string()))
        );
        assert_eq!(
            output.resources.get("배치0").map(value_to_string),
            Some("차림[차림[2, 3], 차림[3, 3], 차림[2, 4], 차림[3, 4]]".to_string())
        );
        assert_eq!(
            output.resources.get("점수4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2100)))
        );
        assert_eq!(
            output.resources.get("줄4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(11)))
        );
        assert_eq!(
            output.resources.get("레벨4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        assert_eq!(
            output.resources.get("고정"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("위치").map(value_to_string),
            Some("차림[1, 1]".to_string())
        );
    }

    #[test]
    fn std_grid_game_view_projects_text_cells_and_summary() {
        let script = r#"
매틱:움직씨 = {
    빈 <- ".".
    격0 <- (4, 4, 빈) 격자.만들기.
    격1 <- (격0, 0, 3, "X") 격자.바꾼값.
    가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
    상태0 <- () 격자게임상태.초기화.
    진행 <- (상태0, "진행") 격자게임상태.바꾸기.
    점 <- () 격자게임점수.초기화.
    오 <- ("O") 테트로미노.만들기.
    낙 <- (오, 1, 0) 낙하조각.만들기.
    세션 <- (격1, 가방, 진행, 점, 낙) 격자게임세션.만들기.
    칸목록 <- (세션, 빈, "O") 격자게임보기.칸목록.
    칸수 <- (칸목록) 길이.
    칸0 <- (칸목록, 0) 차림.값.
    칸1 <- (칸목록, 1) 차림.값.
    칸12 <- (칸목록, 12) 차림.값.
    원천0 <- 칸0.원천.
    원천1 <- 칸1.원천.
    원천12 <- 칸12.원천.
    값1 <- 칸1.값.
    문자판 <- (세션, 빈, "O") 격자게임보기.문자판.
    요약 <- (세션) 격자게임보기.상태요약.
    요약종류 <- 요약.__종류.
    요약틱 <- 요약.틱.
    요약점수 <- 요약.점수.
    요약줄수 <- 요약.줄수.
    요약레벨 <- 요약.레벨.
    요약위치 <- 요약.낙하조각위치.
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_view.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "칸수".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        for key in ["원천0", "원천1", "원천12", "값1", "문자판", "요약종류"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        for key in ["요약틱", "요약점수", "요약줄수", "요약레벨"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        defaults.insert("요약위치".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("칸수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(16)))
        );
        assert_eq!(
            output.resources.get("원천0"),
            Some(&RuntimeValue::String("빈칸".to_string()))
        );
        assert_eq!(
            output.resources.get("원천1"),
            Some(&RuntimeValue::String("낙하".to_string()))
        );
        assert_eq!(
            output.resources.get("원천12"),
            Some(&RuntimeValue::String("고정".to_string()))
        );
        assert_eq!(
            output.resources.get("값1"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("문자판"),
            Some(&RuntimeValue::String(".OO.\n.OO.\n....\nX...".to_string()))
        );
        assert_eq!(
            output.resources.get("요약종류"),
            Some(&RuntimeValue::String(
                "std_grid_game_view_summary".to_string()
            ))
        );
        assert_eq!(
            output.resources.get("요약틱"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("요약점수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("요약줄수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("요약레벨"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("요약위치").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("요약") else {
            panic!("요약 must be pack");
        };
        assert_eq!(
            fields.get("상태"),
            Some(&RuntimeValue::String("진행".to_string()))
        );
    }

    #[test]
    fn std_grid_game_bogae_projects_rect_drawlist_and_size() {
        let script = r##"
매틱:움직씨 = {
    빈 <- ".".
    격0 <- (4, 4, 빈) 격자.만들기.
    격1 <- (격0, 0, 3, "X") 격자.바꾼값.
    가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
    상태0 <- () 격자게임상태.초기화.
    진행 <- (상태0, "진행") 격자게임상태.바꾸기.
    점 <- () 격자게임점수.초기화.
    오 <- ("O") 테트로미노.만들기.
    낙 <- (오, 1, 0) 낙하조각.만들기.
    세션 <- (격1, 가방, 진행, 점, 낙) 격자게임세션.만들기.
    목록 <- (세션, 빈, "O", 8) 격자게임보기.보개목록.
    크기 <- (세션, 8) 격자게임보기.보개크기.
    칸수 <- (목록) 길이.
    첫 <- (목록, 0) 차림.값.
    둘 <- (목록, 1) 차림.값.
    고정 <- (목록, 12) 차림.값.
    첫id <- 첫.id.
    첫결 <- 첫.결.
    첫x <- 첫.x.
    첫색 <- 첫.채움색.
    둘id <- 둘.id.
    둘x <- 둘.x.
    둘색 <- 둘.채움색.
    고정id <- 고정.id.
    고정y <- 고정.y.
    고정색 <- 고정.채움색.
    가로 <- 크기.가로.
    세로 <- 크기.세로.
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_game_bogae.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "칸수".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        for key in ["첫id", "첫결", "첫색", "둘id", "둘색", "고정id", "고정색"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        for key in ["첫x", "둘x", "고정y", "가로", "세로"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("칸수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(16)))
        );
        assert_eq!(
            output.resources.get("첫id"),
            Some(&RuntimeValue::String("격자게임셀_0_0".to_string()))
        );
        assert_eq!(
            output.resources.get("첫결"),
            Some(&RuntimeValue::String("#보개/2D.Rect".to_string()))
        );
        assert_eq!(
            output.resources.get("첫x"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("첫색"),
            Some(&RuntimeValue::String("#111111ff".to_string()))
        );
        assert_eq!(
            output.resources.get("둘id"),
            Some(&RuntimeValue::String("격자게임셀_0_1".to_string()))
        );
        assert_eq!(
            output.resources.get("둘x"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(8)))
        );
        assert_eq!(
            output.resources.get("둘색"),
            Some(&RuntimeValue::String("#ffcc00ff".to_string()))
        );
        assert_eq!(
            output.resources.get("고정id"),
            Some(&RuntimeValue::String("격자게임셀_3_0".to_string()))
        );
        assert_eq!(
            output.resources.get("고정y"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(24)))
        );
        assert_eq!(
            output.resources.get("고정색"),
            Some(&RuntimeValue::String("#4a90e2ff".to_string()))
        );
        assert_eq!(
            output.resources.get("가로"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(32)))
        );
        assert_eq!(
            output.resources.get("세로"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(32)))
        );
    }

    #[test]
    fn std_grid_game_rules_hold_ghost_and_wall_kick_helpers() {
        let script = r##"
매틱:움직씨 = {
    빈 <- ".".
    막힘 <- ("X") 차림.
    격 <- (4, 6, 빈) 격자.만들기.
    가방0 <- (0, ("I", "O") 차림) 무작위가방.만들기.
    상태0 <- () 격자게임상태.초기화.
    진행 <- (상태0, "진행") 격자게임상태.바꾸기.
    점 <- () 격자게임점수.초기화.
    티 <- ("T") 테트로미노.만들기.
    오 <- ("O") 테트로미노.만들기.
    낙T <- (티, 0, 1) 낙하조각.만들기.
    낙O <- (오, 1, 0) 낙하조각.만들기.
    홀드0 <- () 격자게임홀드.초기화.
    안씀 <- (홀드0) 격자게임홀드.썼나.
    교체1 <- (홀드0, 낙T, 가방0, 2, 0) 격자게임홀드.교체.
    홀드1 <- 교체1.홀드.
    홀드1칸 <- (홀드1) 격자게임홀드.칸.
    홀드1씀 <- (홀드1) 격자게임홀드.썼나.
    낙1 <- 교체1.낙하조각.
    낙1위치 <- (낙1) 낙하조각.위치.
    홀드2준비 <- (홀드1) 격자게임홀드.초기화턴.
    홀드2씀 <- (홀드2준비) 격자게임홀드.썼나.
    세션 <- (격, 가방0, 진행, 점, 낙O) 격자게임세션.만들기.
    유령 <- (세션, 막힘) 격자게임보기.유령조각.
    유령위치 <- (유령) 낙하조각.위치.
    유령보개 <- (세션, 빈, "O", "G", 4, 막힘) 격자게임보기.유령보개목록.
    낙하칸 <- (유령보개, 1) 차림.값.
    유령칸 <- (유령보개, 17) 차림.값.
    낙하색 <- 낙하칸.채움색.
    유령색 <- 유령칸.채움색.
    회전 <- (낙T, 격, 막힘, "오른쪽") 격자게임.회전시도.
    회전성공 <- 회전.성공.
    회전오프셋 <- 회전.오프셋.
    회전낙 <- 회전.낙하조각.
    회전위치 <- (회전낙) 낙하조각.위치.
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_game_rules.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["홀드1칸", "낙1위치", "유령위치", "회전오프셋", "회전위치"] {
            defaults.insert(key.to_string(), RuntimeValue::None);
        }
        for key in ["안씀", "홀드1씀", "홀드2씀", "회전성공"] {
            defaults.insert(key.to_string(), RuntimeValue::Bool(false));
        }
        for key in ["낙하색", "유령색"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("안씀"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("홀드1칸").map(value_to_string),
            Some("(__종류: std_block_piece, 칸들: 차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[0, 1]])".to_string())
        );
        assert_eq!(
            output.resources.get("홀드1씀"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("낙1위치").map(value_to_string),
            Some("차림[2, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("홀드2씀"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("유령위치").map(value_to_string),
            Some("차림[1, 4]".to_string())
        );
        assert_eq!(
            output.resources.get("낙하색"),
            Some(&RuntimeValue::String("#ffcc00ff".to_string()))
        );
        assert_eq!(
            output.resources.get("유령색"),
            Some(&RuntimeValue::String("#88ffffff".to_string()))
        );
        assert_eq!(
            output.resources.get("회전성공"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("회전오프셋").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("회전위치").map(value_to_string),
            Some("차림[1, 1]".to_string())
        );
    }

    #[test]
    fn std_unit_closure_temperature_smoke() {
        let script = r#"
매틱:움직씨 = {
    온도동치 <- 25@C == 77@F.
    온도차이확인 <- (30@C - 20@C) == 10@K.
}
"#;
        let program = DdnProgram::from_source(script, "std_unit_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("온도동치".to_string(), RuntimeValue::Bool(false));
        defaults.insert("온도차이확인".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        for key in ["온도동치", "온도차이확인"] {
            match output.resources.get(key) {
                Some(RuntimeValue::Bool(value)) => assert!(*value, "{key}"),
                other => panic!("{key} must be bool, got {:?}", other),
            }
        }
    }

    #[test]
    fn std_input_map_closure_runtime_maps_explicit_and_default_keys() {
        let explicit_map = make_std_input_map(BTreeMap::from([
            (
                "왼쪽".to_string(),
                RuntimeValue::String("ArrowLeft".to_string()),
            ),
            (
                "오른쪽".to_string(),
                RuntimeValue::String("ArrowRight".to_string()),
            ),
            (
                "위".to_string(),
                RuntimeValue::String("ArrowUp".to_string()),
            ),
            (
                "아래".to_string(),
                RuntimeValue::String("ArrowDown".to_string()),
            ),
            (
                "확인".to_string(),
                RuntimeValue::String("Space".to_string()),
            ),
        ]));
        let explicit_fields = std_input_map_fields(&explicit_map).expect("input map fields");
        let input_state = InputState::new(KEY_D, 0);
        assert!(input_map_action_pressed(
            &input_state,
            &explicit_fields,
            "오른쪽"
        ));
        assert!(!input_map_action_pressed(
            &input_state,
            &explicit_fields,
            "확인"
        ));

        let script = r#"
매틱:움직씨 = {
    기본 <- () 입력사상.만들기.
    방향 <- (기본) 입력사상.방향.
    오른쪽 <- (기본, "오른쪽") 입력사상.동작.
    확인 <- (기본, "확인") 입력사상.동작.
}
"#;
        let program = DdnProgram::from_source(script, "std_input_map_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut input = empty_input();
        input.keys_pressed = KEY_D;
        input.last_key_name = "ArrowRight".to_string();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("방향".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("오른쪽".to_string(), RuntimeValue::Bool(false));
        defaults.insert("확인".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &input, &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("방향").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("오른쪽"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("확인"),
            Some(&RuntimeValue::Bool(false))
        );
    }

    #[test]
    fn template_format_can_render_temperature_in_celsius_and_fahrenheit() {
        let script = r#"
매틱:움직씨 = {
    섭씨텍스트 <- (t=298.15@K) 글무늬{"섭씨={t|@.1C}"}.
    화씨텍스트 <- (t=298.15@K) 글무늬{"화씨={t|@.1F}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "temperature_template_unit.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "섭씨텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert(
            "화씨텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("섭씨텍스트") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "섭씨=25.0@C"),
            other => panic!("섭씨텍스트 must be string, got {:?}", other),
        }
        match output.resources.get("화씨텍스트") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "화씨=77.0@F"),
            other => panic!("화씨텍스트 must be string, got {:?}", other),
        }
    }

    #[test]
    fn template_format_defaults_temperature_display_to_kelvin() {
        let script = r#"
매틱:움직씨 = {
    기본텍스트 <- (t=25@C) 글무늬{"온도={t}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "temperature_template_default.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "기본텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("기본텍스트") {
            Some(RuntimeValue::String(value)) => {
                assert!(value.ends_with("@K"), "unexpected value: {}", value)
            }
            other => panic!("기본텍스트 must be string, got {:?}", other),
        }
    }

    #[test]
    fn regex_is_age3_feature_gated() {
        let script = r#"
매틱:움직씨 = {
    결과 <- ("a1", 정규식{"[0-9]+"}) 정규찾기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age2);
        let err = match DdnProgram::from_source(script, "regex_age2.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate code, got {err}"
        );
        assert!(
            err.contains("regex_api"),
            "expected regex feature, got {err}"
        );
    }

    #[test]
    fn quantifier_is_age4_feature_gated() {
        let script = r#"
매틱:움직씨 = {
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let err = match DdnProgram::from_source(script, "quantifier_age3.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate code, got {err}"
        );
        assert!(
            err.contains("logic_quantifier"),
            "expected quantifier feature, got {err}"
        );
    }

    #[test]
    fn quantifier_statement_runs_as_proof_only_noop() {
        let script = r#"
매틱:움직씨 = {
    결과 <- 3.
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age4);
        let program = DdnProgram::from_source(script, "quantifier_age4.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "결과"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn numeric_family_constructors_render_and_normalize() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("123456789012345678901234567890") 큰바른수.
    나눔 <- (2, 4) 나눔수.
    곱 <- (84) 곱수.
    요약 <- (큰=큰, 나눔=나눔, 곱=곱) 글무늬{"{큰}|{나눔}|{곱}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_ctor.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "123456789012345678901234567890|1/2|2^2 * 3 * 7");
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_alias_constructors_are_supported() {
        let script = r#"
매틱:움직씨 = {
    수값 <- ("3.25") fixed64.
    정 <- ("3") int.
    큰 <- ("9007199254740993") bigint.
    나눔 <- (6, 9) rational.
    곱 <- (72) factor.
    요약 <- (수값=수값, 정=정, 큰=큰, 나눔=나눔, 곱=곱) 글무늬{"{수값}|{정}|{큰}|{나눔}|{곱}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_ctor.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("수값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "3.25|3|9007199254740993|2/3|2^3 * 3^2");
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn relation_solve_surface_parses_and_runs() {
        let script = r#"
매틱:움직씨 = {
    rel <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
    결과 <- (rel) 방정식풀기.
}
"#;
        let program = DdnProgram::from_source(script, "relation_solve.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        assert_eq!(
            fields.get(RELATION_SOLVE_VAR_FIELD),
            Some(&RuntimeValue::String("x".to_string()))
        );
        assert_eq!(output.work_cost.counters.solver_evaluations, 1);
    }

    #[test]
    fn relation_eq_infix_runtime_stores_relation_pack() {
        let script = r#"
매틱:움직씨 = {
    관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
}
"#;
        let program = DdnProgram::from_source(script, "relation_eq_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("관계") else {
            panic!("관계 must be relation pack");
        };
        assert_eq!(
            fields.get(RELATION_KIND_FIELD),
            Some(&RuntimeValue::String(RELATION_KIND_EQUATION.to_string()))
        );
        assert!(matches!(
            fields.get(RELATION_LEFT_FIELD),
            Some(RuntimeValue::Formula(_))
        ));
        assert!(matches!(
            fields.get(RELATION_RIGHT_FIELD),
            Some(RuntimeValue::Formula(_))
        ));
    }

    #[test]
    fn connect_jitgi_runtime_matches_relation_eq_pack() {
        let script = r#"
매틱:움직씨 = {
    잇기관계 <- ((#ascii) 수식{2*x + 3}, (#ascii) 수식{7}) 잇기.
    직접관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
}
"#;
        let program = DdnProgram::from_source(script, "connect_jitgi_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("잇기관계".to_string(), RuntimeValue::None);
        defaults.insert("직접관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(jitgi)) = output.resources.get("잇기관계") else {
            panic!("잇기관계 must be relation pack");
        };
        let Some(RuntimeValue::Pack(direct)) = output.resources.get("직접관계") else {
            panic!("직접관계 must be relation pack");
        };
        assert_eq!(jitgi, direct);
        assert_eq!(
            jitgi.get(RELATION_KIND_FIELD),
            Some(&RuntimeValue::String(RELATION_KIND_EQUATION.to_string()))
        );
    }

    #[test]
    fn connect_endpoint_equal_runtime_stores_endpoint_relation_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}
"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_equal.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("전구.왼핀.전압".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("같게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_flow_runtime_stores_endpoint_flow_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}
"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_flow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint flow pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("전지.양극.전류".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("전구.왼핀.전류".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("흐르게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("전류".to_string()))
        );
        assert_eq!(
            fields.get("부호규약"),
            Some(&RuntimeValue::String("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            fields.get("방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_reverse_flow_runtime_stores_endpoint_flow_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_reverse_flow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint reverse flow pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("가계1.구매끝.돈".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("장터.소매끝.돈".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("거슬러 흐르게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("돈".to_string()))
        );
        assert_eq!(
            fields.get("부호규약"),
            Some(&RuntimeValue::String("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            fields.get("방향"),
            Some(&RuntimeValue::String("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_rejects_carried_property_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_carried.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_carried_property_forward_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_carried_forward.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
        assert_eq!(
            carried.get("왼쪽운반자"),
            Some(&RuntimeValue::String("은행.대출창구.대출금".to_string()))
        );
        assert_eq!(
            carried.get("오른쪽운반자"),
            Some(&RuntimeValue::String("기업1.차입끝.대출금".to_string()))
        );
        assert_eq!(
            carried.get("속성"),
            Some(&RuntimeValue::String("위험".to_string()))
        );
        assert_eq!(
            carried.get("운반채널"),
            Some(&RuntimeValue::String("대출금".to_string()))
        );
        assert_eq!(
            carried.get("운반규칙"),
            Some(&RuntimeValue::String("흐르게".to_string()))
        );
        assert_eq!(
            carried.get("운반방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_carried_property_reverse_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_carried_reverse.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
        assert_eq!(
            carried.get("속성"),
            Some(&RuntimeValue::String("재화".to_string()))
        );
        assert_eq!(
            carried.get("운반채널"),
            Some(&RuntimeValue::String("돈".to_string()))
        );
        assert_eq!(
            carried.get("운반규칙"),
            Some(&RuntimeValue::String("거슬러 흐르게".to_string()))
        );
        assert_eq!(
            carried.get("운반방향"),
            Some(&RuntimeValue::String("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_multi.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽끝"),
            Some(&RuntimeValue::String("전지.양극".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽끝"),
            Some(&RuntimeValue::String("전구.왼핀".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(first) = &relations[0] else {
            panic!("first relation must be a pack");
        };
        let RuntimeValue::Pack(second) = &relations[1] else {
            panic!("second relation must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            first.get("채널"),
            Some(&RuntimeValue::String("전압".to_string()))
        );
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            second.get("채널"),
            Some(&RuntimeValue::String("전류".to_string()))
        );
        assert_eq!(
            second.get("방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_econ_runtime_preserves_source_order() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (체결값은 같게, 재화는 흐르게, 돈은 거슬러 흐르게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_multi_econ.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 3);
        let channels = relations
            .iter()
            .map(|item| {
                let RuntimeValue::Pack(fields) = item else {
                    panic!("relation item must be a pack");
                };
                fields.get("채널").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            channels,
            vec![
                Some(RuntimeValue::String("체결값".to_string())),
                Some(RuntimeValue::String("재화".to_string())),
                Some(RuntimeValue::String("돈".to_string()))
            ]
        );
    }

    #[test]
    fn connect_endpoint_statement_append_same_pair_runtime_stores_statement_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_statement_append_same.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_statement_set".to_string()))
        );
        assert_eq!(
            fields.get("대상"),
            Some(&RuntimeValue::String("이음관계".to_string()))
        );
        let Some(RuntimeValue::List(items)) = fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.len(), 2);
        let RuntimeValue::Pack(first) = &items[0] else {
            panic!("first statement must be a pack");
        };
        let RuntimeValue::Pack(second) = &items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_statement_append_mixed_pair_runtime_stores_statement_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_statement_append_mixed.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        let Some(RuntimeValue::List(items)) = fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.len(), 2);
        let RuntimeValue::Pack(second) = &items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = second.get("관계들") else {
            panic!("mixed second statement 관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
    }

    #[test]
    fn connect_endpoint_normalize_single_runtime_returns_one_relation() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    목록 <- (이음관계) 이음관계.관계목록.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_normalize_single.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("목록".to_string(), RuntimeValue::None);
        defaults.insert("정규".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(list)) = output.resources.get("목록") else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.len(), 1);
        let RuntimeValue::Pack(first) = &list[0] else {
            panic!("first relation must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        let Some(RuntimeValue::Pack(normalized)) = output.resources.get("정규") else {
            panic!("정규 must be a pack");
        };
        assert_eq!(
            normalized.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_relation_flat_set".to_string()
            ))
        );
        assert!(matches!(
            normalized.get("개수"),
            Some(RuntimeValue::Fixed64(value)) if value.int_part() == 1
        ));
    }

    #[test]
    fn connect_endpoint_normalize_statement_set_flattens_nested_relation_set() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
    목록 <- (이음관계) 이음관계.관계목록.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_normalize_statement_set.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("목록".to_string(), RuntimeValue::None);
        defaults.insert("정규".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(list)) = output.resources.get("목록") else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.len(), 3);
        let kinds = list
            .iter()
            .map(|item| {
                let RuntimeValue::Pack(fields) = item else {
                    panic!("relation item must be a pack");
                };
                fields.get("__이음관계종류").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                Some(RuntimeValue::String("endpoint_equality".to_string())),
                Some(RuntimeValue::String("endpoint_flow".to_string())),
                Some(RuntimeValue::String(
                    "endpoint_carried_property".to_string()
                )),
            ]
        );
        let Some(RuntimeValue::Pack(normalized)) = output.resources.get("정규") else {
            panic!("정규 must be a pack");
        };
        assert!(matches!(
            normalized.get("개수"),
            Some(RuntimeValue::Fixed64(value)) if value.int_part() == 3
        ));
        let Some(RuntimeValue::List(relations)) = normalized.get("관계들") else {
            panic!("normalized 관계들 must be a list");
        };
        assert_eq!(relations.len(), 3);
    }

    #[test]
    fn connect_endpoint_formula_relation_bridge_maps_endpoint_paths_to_ascii_vars() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    관계들 <- (이음관계) 이음관계.방정식목록.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_bridge.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("관계들".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(relations)) = output.resources.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(first_relation) = &relations[0] else {
            panic!("first formula relation must be a pack");
        };
        assert_eq!(
            first_relation.get("__관계종류"),
            Some(&RuntimeValue::String("방정식".to_string()))
        );
        let Some(RuntimeValue::Formula(left)) = first_relation.get("왼쪽") else {
            panic!("first left must be formula");
        };
        let Some(RuntimeValue::Formula(right)) = first_relation.get("오른쪽") else {
            panic!("first right must be formula");
        };
        assert_eq!(left.raw, "ep_001");
        assert_eq!(right.raw, "ep_002");

        let Some(RuntimeValue::Pack(set)) = output.resources.get("방정식묶음") else {
            panic!("방정식묶음 must be a pack");
        };
        assert_eq!(
            set.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_formula_relation_set".to_string()
            ))
        );
        let Some(RuntimeValue::List(mappings)) = set.get("변수사상") else {
            panic!("변수사상 must be a list");
        };
        assert_eq!(mappings.len(), 2);
        let RuntimeValue::Pack(first_mapping) = &mappings[0] else {
            panic!("first mapping must be a pack");
        };
        assert_eq!(
            first_mapping.get("변수"),
            Some(&RuntimeValue::String("ep_001".to_string()))
        );
        assert_eq!(
            first_mapping.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_formula_relation_solve_uses_explicit_relation_list() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    관계들 <- 방정식묶음.관계들.
    결과 <- (관계들) 방정식풀기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_solve.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        defaults.insert("관계들".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(result)) = output.resources.get("결과") else {
            panic!("결과 must be a pack");
        };
        assert_eq!(
            result.get("__풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::Pack(bindings)) = result.get("해") else {
            panic!("해 must be a pack");
        };
        assert!(bindings.contains_key("ep_001"));
        assert!(bindings.contains_key("ep_002"));
    }

    #[test]
    fn connect_endpoint_formula_relation_rejects_carried_property_metadata() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_unsupported.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        assert!(runner
            .run_update(&world, &empty_input(), &defaults)
            .is_err());
    }

    #[test]
    fn connect_endpoint_boundary_value_injection_appends_relation_packs() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_boundary_value_injection.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let injected_relation_list =
            eval_endpoint_boundary_value_relation_list(&[formula_set.clone(), value_list.clone()])
                .expect("value relation list");
        let RuntimeValue::List(injected_relations) = &injected_relation_list else {
            panic!("값관계들 must be a list");
        };
        assert_eq!(injected_relations.len(), 1);
        let RuntimeValue::Pack(injected_relation) = &injected_relations[0] else {
            panic!("injected relation must be a pack");
        };
        assert_eq!(
            injected_relation.get("__관계종류"),
            Some(&RuntimeValue::String("방정식".to_string()))
        );
        let Some(RuntimeValue::Formula(left)) = injected_relation.get("왼쪽") else {
            panic!("injected left must be formula");
        };
        let Some(RuntimeValue::Formula(right)) = injected_relation.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(left.raw, "ep_001");
        assert_eq!(right.raw, "5");

        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set, value_list]).expect("inject");
        let RuntimeValue::Pack(set) = &injected_set else {
            panic!("주입묶음 must be a pack");
        };
        assert_eq!(
            set.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_formula_relation_set_with_values".to_string()
            ))
        );
        assert_eq!(
            set.get("개수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        assert_eq!(
            set.get("주입개수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        let Some(RuntimeValue::List(all_relations)) = set.get("관계들") else {
            panic!("combined 관계들 must be a list");
        };
        assert_eq!(all_relations.len(), 2);
    }

    #[test]
    fn connect_endpoint_boundary_value_solve_remap_accepts_injected_formula_set() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_boundary_value_solve.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set.clone(), value_list])
                .expect("inject");
        let RuntimeValue::Pack(injected_fields) = &injected_set else {
            panic!("주입묶음 must be a pack");
        };
        let relations_value = injected_fields.get("관계들").expect("관계들").clone();
        let relations = expect_equation_relations(&[relations_value]).expect("relations");
        let solve = eval_relation_solve_result(&relations).expect("solve");
        let remapped = eval_endpoint_solve_result_remap(&[injected_set, solve]).expect("remap");
        let RuntimeValue::Pack(remap) = &remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_boundary_value_rejects_unsupported_inputs() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_boundary_value_bad_base.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["이음관계", "방정식묶음"] {
            defaults.insert(key.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();

        let duplicate =
            endpoint_boundary_test_values(&[("전지.양극.전압", 5), ("전지.양극.전압", 6)]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set.clone(), duplicate])
            .expect_err("duplicate must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_duplicate_path"),
            "{err}"
        );

        let unknown = endpoint_boundary_test_values(&[("없는.끝.전압", 5)]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set.clone(), unknown])
            .expect_err("unknown must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_unknown_path"),
            "{err}"
        );

        let bad_value = endpoint_boundary_test_values_raw(vec![(
            "전지.양극.전압",
            RuntimeValue::String("5".to_string()),
        )]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set, bad_value])
            .expect_err("non numeric must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_non_numeric"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_equal_value_returns_endpoint_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_equal.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = &remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        let RuntimeValue::Pack(second) = &values[1] else {
            panic!("second value must be a pack");
        };
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
        assert_eq!(second.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_flow_value_returns_opposite_endpoint_value() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_flow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전류", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = &remapped else {
            panic!("원복 must be a pack");
        };
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        let RuntimeValue::Pack(left) = &values[0] else {
            panic!("left value must be a pack");
        };
        let RuntimeValue::Pack(right) = &values[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(left.get("값").map(value_to_string).as_deref(), Some("5"));
        assert_eq!(right.get("값").map(value_to_string).as_deref(), Some("-5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_flat_set_skips_public_normalize_step() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_flat.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "정규"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let flat = output.resources.get("정규").expect("정규").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[flat, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = &remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn connect_endpoint_explicit_solve_rejects_unsupported_and_boundary_errors() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_unsupported.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        assert!(eval_endpoint_explicit_solve(&[relation, RuntimeValue::List(vec![])]).is_err());

        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_duplicate.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let duplicate =
            endpoint_boundary_test_values(&[("전지.양극.전압", 5), ("전지.양극.전압", 6)]);
        let err =
            eval_endpoint_explicit_solve(&[relation, duplicate]).expect_err("duplicate must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_duplicate_path"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_injection_records_unit_metadata() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_unit_boundary_injection.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values_unit(&[("전지.양극.전압", 5, UnitDim::KRW)]);
        let injected_relation_list =
            eval_endpoint_boundary_value_relation_list(&[formula_set.clone(), value_list.clone()])
                .expect("value relation list");
        let RuntimeValue::List(injected_relations) = &injected_relation_list else {
            panic!("값관계들 must be a list");
        };
        let RuntimeValue::Pack(relation) = &injected_relations[0] else {
            panic!("injected relation must be a pack");
        };
        let Some(RuntimeValue::Formula(right)) = relation.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(right.raw, "5");

        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set, value_list]).expect("inject");
        let RuntimeValue::Pack(set) = &injected_set else {
            panic!("주입묶음 must be a pack");
        };
        let Some(RuntimeValue::List(injected)) = set.get("주입값들") else {
            panic!("주입값들 must be a list");
        };
        let RuntimeValue::Pack(first) = &injected[0] else {
            panic!("injected value must be a pack");
        };
        assert_eq!(
            first.get("값").map(value_to_string).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            first.get("단위차원"),
            Some(&RuntimeValue::String("KRW".to_string()))
        );
        assert_eq!(
            first.get("단위기호"),
            Some(&RuntimeValue::String("KRW".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_solve_remap_restores_unit_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_unit_boundary_solve.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values_unit(&[("전지.양극.전류", 5, UnitDim::KRW)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = &remapped else {
            panic!("원복 must be a pack");
        };
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        let RuntimeValue::Pack(left) = &values[0] else {
            panic!("left value must be a pack");
        };
        let RuntimeValue::Pack(right) = &values[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(
            left.get("값").map(value_to_string).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            right.get("값").map(value_to_string).as_deref(),
            Some("-5@KRW")
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_rejects_dim_and_currency_conflicts() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_unit_boundary_conflict.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();

        let dim_values = endpoint_boundary_test_values_unit(&[
            ("전지.양극.전압", 1, UnitDim::LENGTH),
            ("전구.왼핀.전압", 1, UnitDim::FORCE),
        ]);
        let injected = eval_endpoint_boundary_value_injection(&[formula_set.clone(), dim_values])
            .expect("inject dim conflict");
        let err = eval_endpoint_solve_result_remap(&[injected, endpoint_fake_solve_result()])
            .expect_err("dimension conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_unit_boundary_dim_conflict"),
            "{err}"
        );

        let currency_values = endpoint_boundary_test_values_unit(&[
            ("전지.양극.전압", 1, UnitDim::KRW),
            ("전구.왼핀.전압", 1, UnitDim::USD),
        ]);
        let injected = eval_endpoint_boundary_value_injection(&[formula_set, currency_values])
            .expect("inject currency conflict");
        let err = eval_endpoint_solve_result_remap(&[injected, endpoint_fake_solve_result()])
            .expect_err("currency conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_unit_boundary_incompatible_unit"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_check_pass_and_violation() {
        let remap = endpoint_range_solved_result("전압", 5, None);
        let pass = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked =
            eval_endpoint_boundary_range_check(&[remap.clone(), pass]).expect("range check");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_range_check".to_string()))
        );
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(check.get("위반개수"), Some(&fixed_value(0)));

        let fail = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(4))),
        )]);
        let checked = eval_endpoint_boundary_range_check(&[remap, fail]).expect("range check");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("above_max".to_string()))
        );
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_pass_and_fail() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_range_pass.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked =
            eval_endpoint_explicit_solve_range_check(&[relation.clone(), values.clone(), ranges])
                .expect("explicit solve range");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_check".to_string()
            ))
        );
        assert_eq!(
            check.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(check.get("위반개수"), Some(&fixed_value(0)));
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(4))),
        )]);
        let checked =
            eval_endpoint_explicit_solve_range_check(&[relation, values, ranges]).expect("fail");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::Pack(range_check)) = check.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(RuntimeValue::List(violations)) = range_check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("above_max".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_missing_and_error_propagation() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_range_missing.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked = eval_endpoint_explicit_solve_range_check(&[
            relation.clone(),
            RuntimeValue::List(vec![]),
            ranges,
        ])
        .expect("missing value check");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::Pack(range_check)) = check.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(RuntimeValue::List(violations)) = range_check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));

        let duplicate_range = endpoint_range_test_values_raw(vec![
            (
                "전구.왼핀.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                None,
            ),
            (
                "전구.왼핀.전압",
                None,
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
            ),
        ]);
        let err = eval_endpoint_explicit_solve_range_check(&[
            relation,
            endpoint_boundary_test_values(&[("전지.양극.전압", 5)]),
            duplicate_range,
        ])
        .expect_err("duplicate range path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_duplicate_path"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_pass_rows_are_endpoint_ordered() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_report_pass.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation.clone(), values.clone(), ranges.clone()])
                .expect("report");
        let rows =
            eval_endpoint_solve_range_report_rows(&[relation, values, ranges]).expect("rows");
        let RuntimeValue::Pack(report) = &report else {
            panic!("report must be a pack");
        };
        assert_eq!(
            report.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_report".to_string()
            ))
        );
        assert_eq!(
            report.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(report.get("행개수"), Some(&fixed_value(2)));
        assert_eq!(report.get("값개수"), Some(&fixed_value(2)));
        assert_eq!(report.get("누락개수"), Some(&fixed_value(0)));
        let Some(RuntimeValue::List(report_rows)) = report.get("행들") else {
            panic!("행들 must be a list");
        };
        let RuntimeValue::List(row_list) = &rows else {
            panic!("rows must be a list");
        };
        assert_eq!(report_rows.len(), 2);
        assert_eq!(row_list.len(), 2);
        let RuntimeValue::Pack(left) = &report_rows[0] else {
            panic!("left row must be a pack");
        };
        let RuntimeValue::Pack(right) = &report_rows[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.get("범위상태"),
            Some(&RuntimeValue::String("범위없음".to_string()))
        );
        assert_eq!(
            right.get("범위상태"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(right.get("하한").map(value_to_string).as_deref(), Some("0"));
        assert_eq!(
            right.get("상한").map(value_to_string).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_missing_rows_distinguish_range_presence() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_report_missing.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let ranges = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation, RuntimeValue::List(vec![]), ranges])
                .expect("report");
        let RuntimeValue::Pack(report) = &report else {
            panic!("report must be a pack");
        };
        assert_eq!(
            report.get("풀이결과종류"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(
            report.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(report.get("누락개수"), Some(&fixed_value(2)));
        let Some(RuntimeValue::List(rows)) = report.get("행들") else {
            panic!("행들 must be a list");
        };
        let RuntimeValue::Pack(left) = &rows[0] else {
            panic!("left row must be a pack");
        };
        let RuntimeValue::Pack(right) = &rows[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.get("값상태"),
            Some(&RuntimeValue::String("누락".to_string()))
        );
        assert_eq!(
            left.get("범위상태"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = left.get("위반들") else {
            panic!("violations must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));
        assert_eq!(
            right.get("값상태"),
            Some(&RuntimeValue::String("누락".to_string()))
        );
        assert_eq!(
            right.get("범위상태"),
            Some(&RuntimeValue::String("범위없음".to_string()))
        );
        assert_eq!(right.get("위반개수"), Some(&fixed_value(0)));
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_formats_tsv_rows() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_text_report.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation.clone(), values.clone(), ranges.clone()])
                .expect("report");
        let text = eval_endpoint_solve_range_text_report(&[report]).expect("text");
        let direct = eval_endpoint_explicit_solve_range_text_report(&[relation, values, ranges])
            .expect("direct text");
        let RuntimeValue::String(text) = &text else {
            panic!("text report must be a string");
        };
        let RuntimeValue::String(direct) = &direct else {
            panic!("direct text report must be a string");
        };
        assert_eq!(text, direct);
        assert!(text.starts_with("변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반"));
        assert!(text.contains("ep_001\t전지.양극.전압\t값있음\t5\t범위없음\t\t\t"));
        assert!(text.contains("ep_002\t전구.왼핀.전압\t값있음\t5\t통과\t0\t10\t"));
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_rejects_non_report_input() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_text_report(&[RuntimeValue::Pack(fields)])
            .expect_err("non report must fail");
        assert!(
            err.to_string()
                .contains("connect_report_text_expected_solve_range_report"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_records_expectation_matrix() {
        let pass_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let pass_program =
            DdnProgram::from_source(pass_script, "connect_endpoint_case_suite_pass.ddn")
                .expect("parse");
        let mut pass_runner = DdnRunner::new(pass_program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let pass_output = pass_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let pass_relation = pass_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let fail_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let fail_program =
            DdnProgram::from_source(fail_script, "connect_endpoint_case_suite_fail.ddn")
                .expect("parse");
        let mut fail_runner = DdnRunner::new(fail_program, "매틱");
        let fail_output = fail_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let fail_relation = fail_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let pass_values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let pass_ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let fail_values = endpoint_boundary_test_values(&[("전지.양극.전류", 5)]);
        let fail_ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전류",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);

        fn case(
            name: &str,
            relation: RuntimeValue,
            values: RuntimeValue,
            ranges: RuntimeValue,
            expected: Option<&str>,
        ) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert("이음관계".to_string(), relation);
            fields.insert("값들".to_string(), values);
            fields.insert("범위들".to_string(), ranges);
            if let Some(expected) = expected {
                fields.insert(
                    "기대검사결과".to_string(),
                    RuntimeValue::String(expected.to_string()),
                );
            }
            RuntimeValue::Pack(fields)
        }

        let work_cost = Rc::new(RefCell::new(WorkCostMeter::default()));
        let suite = eval_endpoint_solve_range_case_suite_with_meter(
            &[RuntimeValue::List(vec![
                case(
                    "pass-default",
                    pass_relation.clone(),
                    pass_values.clone(),
                    pass_ranges.clone(),
                    None,
                ),
                case(
                    "expected-fail",
                    fail_relation.clone(),
                    fail_values.clone(),
                    fail_ranges.clone(),
                    Some("실패"),
                ),
                case(
                    "unexpected-fail",
                    fail_relation,
                    fail_values,
                    fail_ranges,
                    None,
                ),
                case(
                    "unexpected-success",
                    pass_relation,
                    pass_values,
                    pass_ranges,
                    Some("실패"),
                ),
            ])],
            Some(&work_cost),
        )
        .expect("suite");
        assert_eq!(work_cost.borrow().receipt().counters.solver_evaluations, 4);
        let RuntimeValue::Pack(suite_pack) = &suite else {
            panic!("suite must be a pack");
        };
        assert_eq!(
            suite_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite".to_string()
            ))
        );
        assert_eq!(suite_pack.get("전체통과"), Some(&RuntimeValue::Bool(false)));
        assert_eq!(suite_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(suite_pack.get("실패개수"), Some(&fixed_value(2)));
        let text = eval_endpoint_solve_range_case_suite_text(&[suite]).expect("suite text");
        let RuntimeValue::String(text) = &text else {
            panic!("suite text must be a string");
        };
        assert!(text.starts_with("이름\t기대\t실제\t통과"));
        assert!(text.contains("pass-default\t통과\t통과\t참"));
        assert!(text.contains("expected-fail\t실패\t실패\t참"));
        assert!(text.contains("unexpected-fail\t통과\t실패\t거짓"));
        assert!(text.contains("unexpected-success\t실패\t통과\t거짓"));
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_rejects_bad_inputs() {
        let mut bad_expected = BTreeMap::new();
        bad_expected.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_expected.insert("이음관계".to_string(), RuntimeValue::None);
        bad_expected.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        bad_expected.insert("범위들".to_string(), RuntimeValue::List(Vec::new()));
        bad_expected.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("모름".to_string()),
        );
        let err = eval_endpoint_solve_range_case(&[RuntimeValue::Pack(bad_expected)])
            .expect_err("bad expected result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_invalid_expected_result"),
            "{err}"
        );

        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_case_suite_text(&[RuntimeValue::Pack(not_suite)])
            .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_text_expected_suite"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_detail_formats_sections() {
        let pass_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let pass_program =
            DdnProgram::from_source(pass_script, "connect_endpoint_case_suite_detail_pass.ddn")
                .expect("parse");
        let mut pass_runner = DdnRunner::new(pass_program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let pass_output = pass_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let pass_relation = pass_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let flow_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let flow_program =
            DdnProgram::from_source(flow_script, "connect_endpoint_case_suite_detail_flow.ddn")
                .expect("parse");
        let mut flow_runner = DdnRunner::new(flow_program, "매틱");
        let flow_output = flow_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let flow_relation = flow_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        fn case(
            name: &str,
            relation: RuntimeValue,
            values: RuntimeValue,
            ranges: RuntimeValue,
        ) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert("이음관계".to_string(), relation);
            fields.insert("값들".to_string(), values);
            fields.insert("범위들".to_string(), ranges);
            RuntimeValue::Pack(fields)
        }

        let cases = RuntimeValue::List(vec![
            case(
                "voltage-pass",
                pass_relation,
                endpoint_boundary_test_values(&[("전지.양극.전압", 5)]),
                endpoint_range_test_values_raw(vec![(
                    "전구.왼핀.전압",
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
                )]),
            ),
            case(
                "flow-pass",
                flow_relation,
                endpoint_boundary_test_values(&[("전지.양극.전류", 5)]),
                endpoint_range_test_values_raw(vec![(
                    "전구.왼핀.전류",
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(-10))),
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                )]),
            ),
        ]);
        let suite = eval_endpoint_solve_range_case_suite(&[cases.clone()]).expect("suite");
        let detail =
            eval_endpoint_solve_range_case_suite_detail_text(&[suite]).expect("detail text");
        let direct =
            eval_endpoint_solve_range_case_suite_run_detail_text(&[cases]).expect("direct detail");
        let RuntimeValue::String(detail) = &detail else {
            panic!("detail must be a string");
        };
        let RuntimeValue::String(direct) = &direct else {
            panic!("direct detail must be a string");
        };
        assert_eq!(detail, direct);
        assert!(detail.starts_with("이름\t기대\t실제\t통과"));
        assert!(
            detail.contains("\n\n## voltage-pass\n기대\t통과\n실제\t통과\n통과\t참\n변수\t경로")
        );
        assert!(detail.contains("\n\n## flow-pass\n기대\t통과\n실제\t통과\n통과\t참\n변수\t경로"));
        assert!(detail.contains("ep_002\t전구.왼핀.전류\t값있음\t-5\t통과\t-10\t0\t"));
        assert!(!detail.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_detail_rejects_bad_inputs() {
        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err =
            eval_endpoint_solve_range_case_suite_detail_text(&[RuntimeValue::Pack(not_suite)])
                .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_detail_expected_suite"),
            "{err}"
        );

        let mut bad_result = BTreeMap::new();
        bad_result.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
        );
        bad_result.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_result.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert(
            "실제검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert("통과여부".to_string(), RuntimeValue::Bool(true));
        let mut suite = BTreeMap::new();
        suite.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite".to_string()),
        );
        suite.insert(
            "결과들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::Pack(bad_result)]),
        );
        let err = eval_endpoint_solve_range_case_suite_detail_text(&[RuntimeValue::Pack(suite)])
            .expect_err("malformed case result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_detail_malformed_case_result"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_records_mismatch_lists() {
        fn case_result(name: &str, expected: &str, actual: &str, passed: bool) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert(
                "__이음관계종류".to_string(),
                RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
            );
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert(
                "기대검사결과".to_string(),
                RuntimeValue::String(expected.to_string()),
            );
            fields.insert(
                "실제검사결과".to_string(),
                RuntimeValue::String(actual.to_string()),
            );
            fields.insert("통과여부".to_string(), RuntimeValue::Bool(passed));
            RuntimeValue::Pack(fields)
        }

        let suite = endpoint_solve_range_case_suite_pack(
            vec![
                case_result("expected-pass-actual-pass", "통과", "통과", true),
                case_result("expected-fail-actual-fail", "실패", "실패", true),
                case_result("expected-pass-actual-fail", "통과", "실패", false),
                case_result("expected-fail-actual-pass", "실패", "통과", false),
            ],
            2,
        );
        let summary =
            eval_endpoint_solve_range_case_suite_summary(&[suite]).expect("suite summary");
        let RuntimeValue::Pack(summary_pack) = &summary else {
            panic!("summary must be a pack");
        };
        assert_eq!(
            summary_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite_summary".to_string()
            ))
        );
        assert_eq!(summary_pack.get("개수"), Some(&fixed_value(4)));
        assert_eq!(summary_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(summary_pack.get("실패개수"), Some(&fixed_value(2)));
        assert_eq!(
            summary_pack.get("전체통과"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            summary_pack.get("통과케이스들"),
            Some(&RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-pass".to_string()),
                RuntimeValue::String("expected-fail-actual-fail".to_string()),
            ]))
        );
        assert_eq!(
            summary_pack.get("실패케이스들"),
            Some(&RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-fail".to_string()),
                RuntimeValue::String("expected-fail-actual-pass".to_string()),
            ]))
        );
        assert_eq!(
            summary_pack.get("기대실패통과케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string()
            )]))
        );
        assert_eq!(
            summary_pack.get("기대통과실패케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string()
            )]))
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_rejects_bad_inputs() {
        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_case_suite_summary(&[RuntimeValue::Pack(not_suite)])
            .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_summary_expected_suite"),
            "{err}"
        );

        let mut bad_result = BTreeMap::new();
        bad_result.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
        );
        bad_result.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_result.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert(
            "실제검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        let mut suite = BTreeMap::new();
        suite.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite".to_string()),
        );
        suite.insert(
            "결과들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::Pack(bad_result)]),
        );
        let err = eval_endpoint_solve_range_case_suite_summary(&[RuntimeValue::Pack(suite)])
            .expect_err("malformed result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_summary_malformed_case_result"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_records_pass_fail() {
        let mut summary = BTreeMap::new();
        summary.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite_summary".to_string()),
        );
        summary.insert("개수".to_string(), fixed_value(4));
        summary.insert("통과개수".to_string(), fixed_value(2));
        summary.insert("실패개수".to_string(), fixed_value(2));
        summary.insert("전체통과".to_string(), RuntimeValue::Bool(false));
        summary.insert(
            "실패케이스들".to_string(),
            RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-fail".to_string()),
                RuntimeValue::String("expected-fail-actual-pass".to_string()),
            ]),
        );
        summary.insert(
            "기대실패통과케이스들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string(),
            )]),
        );
        summary.insert(
            "기대통과실패케이스들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string(),
            )]),
        );

        let check = eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(summary)])
            .expect("suite check");
        let RuntimeValue::Pack(check_pack) = &check else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite_check".to_string()
            ))
        );
        assert_eq!(
            check_pack.get("판정"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(check_pack.get("개수"), Some(&fixed_value(4)));
        assert_eq!(check_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(check_pack.get("실패개수"), Some(&fixed_value(2)));
        assert_eq!(check_pack.get("전체통과"), Some(&RuntimeValue::Bool(false)));
        assert_eq!(
            check_pack.get("기대실패통과케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string()
            )]))
        );
        assert_eq!(
            check_pack.get("기대통과실패케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string()
            )]))
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_rejects_bad_inputs() {
        let mut not_summary = BTreeMap::new();
        not_summary.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(not_summary)])
            .expect_err("non summary must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_check_expected_summary"),
            "{err}"
        );

        let mut malformed = BTreeMap::new();
        malformed.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite_summary".to_string()),
        );
        malformed.insert("전체통과".to_string(), RuntimeValue::Bool(true));
        let err = eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(malformed)])
            .expect_err("malformed summary must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_check_malformed_summary"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_missing_value_is_soft_violation() {
        let remap = endpoint_partial_range_solve_result();
        let range = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked = eval_endpoint_boundary_range_check(&[remap, range]).expect("range check");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));
        assert_eq!(first.get("하한").map(value_to_string).as_deref(), Some("0"));
        assert_eq!(
            first.get("상한").map(value_to_string).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_unit_policy_and_errors() {
        let remap = endpoint_range_solved_result("전류", 5, Some(UnitDim::KRW));
        let pass = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전류",
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(-10),
                dim: UnitDim::KRW,
            })),
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(0),
                dim: UnitDim::KRW,
            })),
        )]);
        let checked =
            eval_endpoint_boundary_range_check(&[remap.clone(), pass]).expect("range check");
        let RuntimeValue::Pack(check) = &checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );

        let incompatible = endpoint_range_test_values_raw(vec![(
            "전지.양극.전류",
            None,
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(10),
                dim: UnitDim::USD,
            })),
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), incompatible])
            .expect_err("incompatible unit must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_incompatible_unit"),
            "{err}"
        );

        let dim_conflict = endpoint_range_test_values_raw(vec![(
            "전지.양극.전류",
            None,
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(10),
                dim: UnitDim::LENGTH,
            })),
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap, dim_conflict])
            .expect_err("dimension conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_dim_conflict"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_rejects_unsupported_inputs() {
        let remap = endpoint_range_solved_result("전압", 5, None);
        let duplicate = endpoint_range_test_values_raw(vec![
            (
                "전지.양극.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                None,
            ),
            (
                "전지.양극.전압",
                None,
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
            ),
        ]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), duplicate])
            .expect_err("duplicate path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_duplicate_path"),
            "{err}"
        );

        let unknown = endpoint_range_test_values_raw(vec![(
            "없는.끝.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            None,
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), unknown])
            .expect_err("unknown path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_unknown_path"),
            "{err}"
        );

        let mut bad = BTreeMap::new();
        bad.insert(
            "경로".to_string(),
            RuntimeValue::String("전지.양극.전압".to_string()),
        );
        bad.insert("최소".to_string(), RuntimeValue::String("0".to_string()));
        let err = eval_endpoint_boundary_range_check(&[
            remap,
            RuntimeValue::List(vec![RuntimeValue::Pack(bad)]),
        ])
        .expect_err("non numeric must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_non_numeric"),
            "{err}"
        );
    }

    fn endpoint_boundary_test_values(items: &[(&str, i64)]) -> RuntimeValue {
        endpoint_boundary_test_values_raw(
            items
                .iter()
                .map(|(path, value)| (*path, RuntimeValue::Fixed64(Fixed64::from_i64(*value))))
                .collect(),
        )
    }

    fn endpoint_range_solved_result(
        channel: &str,
        left_value: i64,
        unit: Option<UnitDim>,
    ) -> RuntimeValue {
        let left_path = format!("전지.양극.{channel}");
        let right_path = format!("전구.왼핀.{channel}");
        let left = match unit {
            Some(dim) => RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(left_value),
                dim,
            }),
            None => RuntimeValue::Fixed64(Fixed64::from_i64(left_value)),
        };
        let right = match unit {
            Some(dim) => RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(-left_value),
                dim,
            }),
            None => RuntimeValue::Fixed64(Fixed64::from_i64(left_value)),
        };
        endpoint_range_solve_result_from_values(vec![
            ("ep_001", &left_path, Some(left)),
            ("ep_002", &right_path, Some(right)),
        ])
    }

    fn endpoint_partial_range_solve_result() -> RuntimeValue {
        endpoint_range_solve_result_from_values(vec![
            (
                "ep_001",
                "전지.양극.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(5))),
            ),
            ("ep_002", "전구.왼핀.전압", None),
        ])
    }

    fn endpoint_range_solve_result_from_values(
        entries: Vec<(&str, &str, Option<RuntimeValue>)>,
    ) -> RuntimeValue {
        let mut mappings = Vec::new();
        let mut values = Vec::new();
        let mut missing = Vec::new();
        for (variable, path, value) in entries {
            let mut mapping = BTreeMap::new();
            mapping.insert(
                "변수".to_string(),
                RuntimeValue::String(variable.to_string()),
            );
            mapping.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
            mappings.push(RuntimeValue::Pack(mapping));
            if let Some(value) = value {
                let mut item = BTreeMap::new();
                item.insert(
                    "변수".to_string(),
                    RuntimeValue::String(variable.to_string()),
                );
                item.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                item.insert("값".to_string(), value);
                values.push(RuntimeValue::Pack(item));
            } else {
                missing.push(RuntimeValue::String(variable.to_string()));
            }
        }
        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_result".to_string()),
        );
        fields.insert(
            "풀이결과종류".to_string(),
            RuntimeValue::String(if missing.is_empty() {
                "성공".to_string()
            } else {
                "부분성공".to_string()
            }),
        );
        fields.insert("값들".to_string(), RuntimeValue::List(values));
        fields.insert("누락변수들".to_string(), RuntimeValue::List(missing));
        fields.insert("변수사상".to_string(), RuntimeValue::List(mappings));
        fields.insert("원래풀이".to_string(), endpoint_fake_solve_result());
        RuntimeValue::Pack(fields)
    }

    fn endpoint_range_test_values_raw(
        items: Vec<(&str, Option<RuntimeValue>, Option<RuntimeValue>)>,
    ) -> RuntimeValue {
        RuntimeValue::List(
            items
                .into_iter()
                .map(|(path, min, max)| {
                    let mut fields = BTreeMap::new();
                    fields.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                    if let Some(min) = min {
                        fields.insert("최소".to_string(), min);
                    }
                    if let Some(max) = max {
                        fields.insert("최대".to_string(), max);
                    }
                    RuntimeValue::Pack(fields)
                })
                .collect(),
        )
    }

    fn endpoint_fake_solve_result() -> RuntimeValue {
        let mut bindings = BTreeMap::new();
        bindings.insert("ep_001".to_string(), fixed_value(1));
        bindings.insert("ep_002".to_string(), fixed_value(1));
        let mut result = BTreeMap::new();
        result.insert(
            RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
            RuntimeValue::String(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
        );
        result.insert(
            RELATION_SOLVE_BINDINGS_FIELD.to_string(),
            RuntimeValue::Pack(bindings),
        );
        RuntimeValue::Pack(result)
    }

    fn endpoint_boundary_test_values_unit(items: &[(&str, i64, UnitDim)]) -> RuntimeValue {
        endpoint_boundary_test_values_raw(
            items
                .iter()
                .map(|(path, value, dim)| {
                    (
                        *path,
                        RuntimeValue::Unit(UnitValue {
                            value: Fixed64::from_i64(*value),
                            dim: *dim,
                        }),
                    )
                })
                .collect(),
        )
    }

    fn endpoint_boundary_test_values_raw(items: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
        RuntimeValue::List(
            items
                .into_iter()
                .map(|(path, value)| {
                    let mut fields = BTreeMap::new();
                    fields.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                    fields.insert("값".to_string(), value);
                    RuntimeValue::Pack(fields)
                })
                .collect(),
        )
    }

    #[test]
    fn connect_endpoint_solve_result_remap_success_returns_endpoint_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    관계들 <- 방정식묶음.관계들.
    풀이 <- (관계들) 방정식풀기.
    원복 <- (방정식묶음, 풀이) 이음관계.풀이원복.
    값들 <- (방정식묶음, 풀이) 이음관계.풀이값목록.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_success.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "관계들", "풀이", "원복", "값들"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(remap)) = output.resources.get("원복") else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.get("변수"),
            Some(&RuntimeValue::String("ep_001".to_string()))
        );
        assert_eq!(
            first.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        let Some(RuntimeValue::List(value_list)) = output.resources.get("값들") else {
            panic!("풀이값목록 must be a list");
        };
        assert_eq!(value_list.len(), 2);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_partial_records_missing_variables() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    부분관계 <- ((#ascii) 수식{ep_001}) =:= ((#ascii) 수식{0}).
    부분풀이 <- (부분관계) 방정식풀기.
    원복 <- (방정식묶음, 부분풀이) 이음관계.풀이원복.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_partial.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "부분관계", "부분풀이", "원복"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(remap)) = output.resources.get("원복") else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("부분성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 1);
        let Some(RuntimeValue::List(missing)) = remap.get("누락변수들") else {
            panic!("누락변수들 must be a list");
        };
        assert_eq!(missing, &vec![RuntimeValue::String("ep_002".to_string())]);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_rejects_unknown_solver_binding() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    나쁜관계 <- ((#ascii) 수식{ep_999}) =:= ((#ascii) 수식{0}).
    나쁜풀이 <- (나쁜관계) 방정식풀기.
    원복 <- (방정식묶음, 나쁜풀이) 이음관계.풀이원복.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_bad.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "나쁜관계", "나쁜풀이", "원복"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        assert!(runner
            .run_update(&world, &empty_input(), &defaults)
            .is_err());
    }

    #[test]
    fn connect_endpoint_rejects_unsupported_multi_inner_sentence_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_multi_unsupported.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_duplicate_carrier_flow_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 흐르게, 돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_duplicate_carrier.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_empty_multi_inner_sentence_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, ) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_multi_empty.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_whole_object_shorthand_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전구와 전지를 (전압은 같게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_unsupported.ddn").is_err());
    }

    #[test]
    fn relation_solve_surface_handles_quadratic_single_root() {
        let script = r#"
매틱:움직씨 = {
    rel <- ((#ascii) 수식{x^2 - 4*x + 4}) =:= ((#ascii) 수식{0}).
    결과 <- (rel) 방정식풀기.
}
"#;
        let program =
            DdnProgram::from_source(script, "relation_solve_quadratic.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        assert_eq!(
            fields.get(RELATION_SOLVE_VALUE_FIELD),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(2)))
        );
    }

    #[test]
    fn relation_solve_surface_handles_linear_system_2x2() {
        let script = r#"
매틱:움직씨 = {
    rel1 <- ((#ascii) 수식{x + y}) =:= ((#ascii) 수식{5}).
    rel2 <- ((#ascii) 수식{x - y}) =:= ((#ascii) 수식{1}).
    rels <- (rel1, rel2) 차림.
    결과 <- (rels) 방정식풀기.
}
"#;
        let program = DdnProgram::from_source(script, "relation_solve_system.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel1".to_string(), RuntimeValue::None);
        defaults.insert("rel2".to_string(), RuntimeValue::None);
        defaults.insert("rels".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        let Some(RuntimeValue::Pack(bindings)) = fields.get(RELATION_SOLVE_BINDINGS_FIELD) else {
            panic!("결과.해 must be pack");
        };
        assert_eq!(
            bindings.get("x"),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(3)))
        );
        assert_eq!(
            bindings.get("y"),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(2)))
        );
    }

    #[test]
    fn butbak_decl_reassignment_fails_in_runtime() {
        let script = r#"
채비 {
    상수:수 = 1.
}.

상수 <- 2.
"#;
        let program = DdnProgram::from_source(script, "butbak_reassign_fail.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("butbak reassignment must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("붙박이는 재대입할 수 없습니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn non_butbak_decl_reassignment_still_runs_in_runtime() {
        let script = r#"
채비 {
    점수:수 <- 0.
}.

점수 <- 1.
"#;
        let program =
            DdnProgram::from_source(script, "non_butbak_reassign_runs.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "점수"),
            Fixed64::from_i64(1)
        );
    }

    #[test]
    fn numeric_family_sized_variant_aliases_are_supported() {
        let script = r#"
매틱:움직씨 = {
    a <- ("1.5") 셈수4.
    b <- ("7") 바른수8.
    c <- (a) 셈수2.
    d <- (b) 바른수4.
    요약 <- (c=c, d=d) 글무늬{"{c}|{d}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_sized_variant_alias.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("b".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("c".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("d".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "1.5|7");
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_sized_variant_type_pins_in_decl_block_are_supported() {
        let script = r#"
채비 {
    값:셈수2 <- 1.5.
    정:바른수4 <- 7.
}.

매틱:움직씨 = {
    요약 <- (값=값, 정=정) 글무늬{"{값}|{정}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_sized_variant_type_pin.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "1.5|7"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_english_alias_type_pins_in_decl_block_are_supported() {
        let script = r#"
채비 {
    실수형:fixed64 <- 1.5.
    정수형:int64 <- 7.
    큰:bigint <- ("9007199254740993") 큰바른수.
    유리:rational <- (6, 9) 나눔수.
    인수:factorized <- (72) 곱수.
}.

매틱:움직씨 = {
    요약 <- (실수형=실수형, 정수형=정수형, 큰=큰, 유리=유리, 인수=인수)
        글무늬{"{실수형}|{정수형}|{큰}|{유리}|{인수}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_english_alias_type_pin.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("실수형".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정수형".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("유리".to_string(), RuntimeValue::None);
        defaults.insert("인수".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert!(text.starts_with("1.5|7|9007199254740993|2/3|"));
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_fixed64_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:fixed64) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_fixed64.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=셈수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("fixed64"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_int64_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:int64) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1.5:x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_int64.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("int64"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_bigint_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:bigint) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_bigint.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=큰바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("bigint"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_rational_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:rational) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_rational.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("rational"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_factorized_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:factorized) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_factorized.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("factorized"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_big_int_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:big_int) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_big_int.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=큰바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("big_int"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_ratio_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:ratio) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_ratio.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("ratio"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_frac_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:frac) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_frac.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("frac"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_factor_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:factor) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_factor.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("factor"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_primepow_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:primepow) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_primepow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("primepow"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_boolean_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:boolean) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "type_alias_diag_boolean.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=참거짓"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("boolean"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_list_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:list) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_list.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=차림"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("list"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_set_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:set) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_set.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=모음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("set"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_map_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:map) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_map.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=짝맞춤"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("map"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_pack_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:pack) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=묶음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("pack"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_string_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:string) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_string.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=글"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("string"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_none_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:none) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_none.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=없음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("none"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_non_keyword_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:non) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "type_alias_diag_non_keyword.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=없음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("non"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_non_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:논) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_non.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=참거짓"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("논"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_mokrok_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:목록) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_mokrok.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=차림"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("목록"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_modum_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:모둠) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_modum.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=모음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("모둠"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_geurimpyo_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:그림표) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "type_alias_diag_geurimpyo.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=짝맞춤"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("그림표"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_valuepack_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:값꾸러미) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "type_alias_diag_valuepack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=묶음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("값꾸러미"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_exact_infix_rational_addition() {
        let script = r#"
매틱:움직씨 = {
    a <- (1, 3) 나눔수.
    b <- (1, 6) 나눔수.
    c <- a + b.
    요약 <- (c=c) 글무늬{"{c}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_exact_rational_add.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "1/2"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_bigint_multiplication_and_division() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000000") 큰바른수.
    b <- ("3") 큰바른수.
    mul <- a * b.
    rat <- a / b.
    요약 <- (mul=mul, rat=rat) 글무늬{"{mul}|{rat}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_bigint_ops.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("mul".to_string(), RuntimeValue::None);
        defaults.insert("rat".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "30000000000000000000|10000000000000000000/3")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_bigint_over_i128_boundary() {
        let script = r#"
매틱:움직씨 = {
    a <- ("340282366920938463463374607431768211456") 큰바른수.
    b <- ("1") 큰바른수.
    c <- a + b.
    요약 <- (c=c) 글무늬{"{c}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_bigint_over_i128.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "340282366920938463463374607431768211457")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_rational_accepts_bigint_operands() {
        let script = r#"
매틱:움직씨 = {
    n <- ("680564733841876926926749214863536422912", "2") 나눔수.
    기대 <- ("340282366920938463463374607431768211456", "1") 나눔수.
    같음 <- n == 기대.
    요약 <- (같음=같음) 글무늬{"{같음}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_rational_bigint_operands.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("기대".to_string(), RuntimeValue::None);
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "참"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_constructor_accepts_bigint_within_i64() {
        let script = r#"
매틱:움직씨 = {
    n <- ("84") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{"{표식}|{경로}|{f}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_factor_from_bigint_ok.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "완료|i64|2^2 * 3 * 7"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_route_metrics_resource_is_emitted() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    a <- ("84") 큰바른수.
    fa <- (a) 곱수.
    b <- ("{huge}") 큰바른수.
    fb <- (b) 곱수.
    c <- ("16000000064000000063") 큰바른수.
    fc <- (c) 곱수.
}}
"#
        );
        let program = DdnProgram::from_source(&script, "numeric_family_factor_route_metrics.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("fa".to_string(), RuntimeValue::None);
        defaults.insert("fb".to_string(), RuntimeValue::None);
        defaults.insert("fc".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        let summary = match output
            .resources
            .get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY)
        {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("route summary must be string, got {:?}", other),
        };
        assert!(
            summary.contains("i64=1"),
            "summary must include i64 route: {summary}"
        );
        assert!(
            summary.contains("deferred:bitlimit=1"),
            "summary must include deferred bitlimit route: {summary}"
        );
        assert!(
            summary.contains("bigint:pollard=1"),
            "summary must include pollard route: {summary}"
        );
        let total = extract_fixed(&output.resources, NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY);
        assert_eq!(total.int_part(), 3);
        let bits_min = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY);
        let bits_max = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY);
        let bits_sum = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY);
        let policy = match output.resources.get(NUMERIC_FACTOR_POLICY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("factor policy must be string, got {:?}", other),
        };
        println!("numeric_factor_policy={policy}");
        assert!(
            policy.contains("bit_limit=512"),
            "policy must expose bit_limit: {policy}"
        );
        assert!(
            policy.contains("pollard_iters=200000"),
            "policy must expose pollard iters: {policy}"
        );
        assert!(
            policy.contains("fallback_limit=1000000"),
            "policy must expose fallback limit: {policy}"
        );
        assert!(
            policy.contains("small_prime_max=101"),
            "policy must expose small-prime max: {policy}"
        );
        assert_eq!(bits_min.int_part(), 7);
        assert!(bits_max.int_part() > FACTOR_BIGINT_FACTOR_BITS_LIMIT as i64);
        assert!(bits_sum.int_part() >= bits_min.int_part() * 3);
    }

    #[test]
    fn numeric_family_factor_route_metrics_diag_event_is_emitted() {
        let script = r#"
매틱:움직씨 = {
    a <- ("84") 큰바른수.
    fa <- (a) 곱수.
    c <- ("16000000064000000063") 큰바른수.
    fc <- (c) 곱수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_factor_route_metrics_diag.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("fa".to_string(), RuntimeValue::None);
        defaults.insert("fc".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        let summary = match output
            .resources
            .get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY)
        {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("route summary must be string, got {:?}", other),
        };
        let total = extract_fixed(&output.resources, NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY);
        let total_int = total.int_part();
        let policy = match output.resources.get(NUMERIC_FACTOR_POLICY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("factor policy must be string, got {:?}", other),
        };

        let events = numeric_factor_route_summary_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY);
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY)
        );
        let bits_min = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY);
        let bits_max = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY);
        let bits_sum = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY);
        let expected_trace = format!(
            "routes={summary};total={total_int};bit_min={};bit_max={};bit_sum={};policy={policy}",
            bits_min.int_part(),
            bits_max.int_part(),
            bits_sum.int_part()
        );
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some(expected_trace.as_str())
        );
        let expected_message = format!(
            "곱수 분해경로 집계: {summary} (총 {total_int}, 비트 최소={}, 최대={}, 합={}, 정책={policy})",
            bits_min.int_part(),
            bits_max.int_part(),
            bits_sum.int_part()
        );
        assert_eq!(
            events[0].message.as_deref(),
            Some(expected_message.as_str())
        );
    }

    #[test]
    fn numeric_family_factor_constructor_deferred_for_bigint_out_of_i64() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{huge}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    원인 <- f.지연사유.
    경로 <- f.분해경로.
    비트 <- f.분해비트수.
    요약 <- (표식=표식, 원인=원인, 경로=경로, f=f) 글무늬{{"{{표식}}|{{원인}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program =
            DdnProgram::from_source(&script, "numeric_family_factor_from_bigint_overflow.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원인".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("비트".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let expected_summary = format!("지연|비트한계초과|deferred:bitlimit|{huge}");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, expected_summary.as_str()),
            other => panic!("요약 must be string, got {:?}", other),
        }
        let bits = extract_fixed(&output.resources, "비트");
        assert!(bits.int_part() > FACTOR_BIGINT_FACTOR_BITS_LIMIT as i64);
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        let expected_agg_routes = format!("{}=1", FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT);
        let expected_trace = format!(
            "route={};bits={};agg_routes={};agg_total=1",
            FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT,
            bits.int_part(),
            expected_agg_routes
        );
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some(expected_trace.as_str())
        );
        assert_eq!(events[0].reason, NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED);
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT)
        );
        let expected_message = format!("곱수 분해를 지연했습니다(비트한계초과): {huge}");
        assert_eq!(
            events[0].message.as_deref(),
            Some(expected_message.as_str())
        );
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED)
        );
        let span = events[0]
            .source_span
            .as_ref()
            .expect("deferred factor diag must carry source_span");
        assert_eq!(span.file, "numeric_family_factor_from_bigint_overflow.ddn");
        assert!(span.start_line >= 1);
    }

    #[test]
    fn numeric_family_factor_constructor_deferred_for_factor_failure_reason() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1);
        let n = "9223372036854775809";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    원인 <- f.지연사유.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 원인=원인, 경로=경로, f=f) 글무늬{{"{{표식}}|{{원인}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_factor_failed.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원인".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(
                    text,
                    "지연|분해실패|deferred:factorfailed|9223372036854775809"
                )
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some("route=deferred:factorfailed;bits=64;agg_routes=deferred:factorfailed=1;agg_total=1")
        );
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED)
        );
        assert_eq!(
            events[0].message.as_deref(),
            Some("곱수 분해를 지연했습니다(분해실패): 9223372036854775809")
        );
    }

    #[test]
    fn numeric_family_factor_constructor_uses_trial_fallback_when_pollard_disabled() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1_000_000);
        let n = "9223372036854775809";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_trial_fallback.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert!(
                text.starts_with("완료|bigint:mixed|"),
                "unexpected summary: {text}"
            ),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_strips_small_prime_without_pollard_or_fallback() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1);
        let n = "510423550381407695195061911147652317181";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_small_prime_strip.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(
                text,
                "완료|bigint:smallprime|3 * 170141183460469231731687303715884105727"
            ),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_uses_pollard_route_for_large_semiprime() {
        let n = "16000000064000000063";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_pollard_route.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "완료|bigint:pollard|4000000007 * 4000000009")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_deferred_diag_keeps_source_span_via_eval_callable() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r##"
매틱:움직씨 = {{
    함수 <- "#곱수".
    값들 <- ("{huge}") 차림.
    결과 <- (값들, 함수) 변환.
    첫 <- (결과) 첫번째.
    표식 <- 첫.분해상태.
    요약 <- (표식=표식) 글무늬{{"{{표식}}"}}.
}}
"##
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_deferred_eval_callable_span.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("함수".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값들".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        defaults.insert("첫".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "지연"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref())
                .map(|text| {
                    text.starts_with("route=deferred:bitlimit;bits=")
                        && text.contains(";agg_routes=deferred:bitlimit=1;agg_total=1")
                })
                .unwrap_or(false),
            "deferred diag must expose route/bits/agg in expr.text"
        );
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT)
        );
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED)
        );
        let span = events[0]
            .source_span
            .as_ref()
            .expect("eval_callable path must keep source_span");
        assert_eq!(
            span.file,
            "numeric_family_factor_deferred_eval_callable_span.ddn"
        );
        assert!(span.start_line >= 1);
    }

    #[test]
    fn numeric_family_i64_path_accepts_bigint_in_range() {
        let script = r#"
매틱:움직씨 = {
    최소 <- ("1") 큰바른수.
    최대 <- ("3") 큰바른수.
    값 <- (최소, 최대) 무작위정수.
    유효 <- (값 >= 최소) 그리고 (값 <= 최대).
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_i64_path_in_range.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("최소".to_string(), RuntimeValue::None);
        defaults.insert("최대".to_string(), RuntimeValue::None);
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("유효".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("유효") {
            Some(RuntimeValue::Bool(flag)) => assert!(*flag),
            other => panic!("유효 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_deferred_value_participates_exact_infix() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{huge}") 큰바른수.
    f <- (n) 곱수.
    next <- f + ("1") 큰바른수.
    요약 <- (next=next) 글무늬{{"{{next}}"}}.
}}
"#
        );
        let program =
            DdnProgram::from_source(&script, "numeric_family_factor_deferred_exact_infix.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("next".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let expected_next = format!("1{}1", "0".repeat(159));
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, expected_next.as_str()),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_constructor_handles_bigint_over_i128_when_prime() {
        let script = r#"
매틱:움직씨 = {
    n <- ("170141183460469231731687303715884105727") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    요약 <- (표식=표식, f=f) 글무늬{"{표식}|{f}"}.
}
"#;
        let program = DdnProgram::from_source(
            script,
            "numeric_family_factor_from_bigint_over_i128_prime.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "완료|170141183460469231731687303715884105727")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_handles_bigint_over_i128_when_factorable() {
        let script = r#"
매틱:움직씨 = {
    n <- ("1361129467683753853853498429727072845824") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    요약 <- (표식=표식, f=f) 글무늬{"{표식}|{f}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_factor_from_bigint_over_i128.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "완료|2^130"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_i64_path_rejects_bigint_out_of_range() {
        let script = r#"
매틱:움직씨 = {
    최소 <- ("9223372036854775808") 큰바른수.
    최대 <- ("9223372036854775810") 큰바른수.
    값 <- (최소, 최대) 무작위정수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_i64_path_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("최소".to_string(), RuntimeValue::None);
        defaults.insert("최대".to_string(), RuntimeValue::None);
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 값(i64 범위)이 필요합니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn numeric_family_exact_infix_comparison_and_equality() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000000") 큰바른수.
    b <- ("9999999999999999999") 큰바른수.
    r1 <- (1, 3) 나눔수.
    r2 <- (2, 5) 나눔수.
    같음 <- (2, 1) 나눔수 == (4, 2) 나눔수.
    큼 <- a > b.
    작음 <- r1 < r2.
    요약 <- (같음=같음, 큼=큼, 작음=작음) 글무늬{"{같음}|{큼}|{작음}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_compare.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("r1".to_string(), RuntimeValue::None);
        defaults.insert("r2".to_string(), RuntimeValue::None);
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("큼".to_string(), RuntimeValue::Bool(false));
        defaults.insert("작음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "참|참|참"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_modulo_for_integer_family() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000003") 큰바른수.
    b <- ("7") 큰바른수.
    m <- a % b.
    요약 <- (m=m) 글무늬{"{m}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_modulo.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("m".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "6"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_integer_types_reject_fractional_value() {
        let script = r#"
매틱:움직씨 = {
    정 <- (1.5) 바른수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_int_reject.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 값이 필요합니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn numeric_family_grouped_integer_strings_are_supported() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("9_223_372_036_854_775_808") 큰바른수.
    다음 <- 큰 + ("1") 큰바른수.
    나눔 <- ("1_000", "4") 나눔수.
    곱 <- ("84_000") 곱수.
    요약 <- (다음=다음, 나눔=나눔, 곱=곱) 글무늬{"{다음}|{나눔}|{곱}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_grouped_integer_strings.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("다음".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "9223372036854775809|250/1|2^5 * 3 * 5^3 * 7")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_grouped_integer_string_rejects_invalid_format() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("1__000") 큰바른수.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_grouped_integer_invalid.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 문자열 형식이 아닙니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn decl_block_typecheck_rejects_mismatched_exact_numeric_initializer() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:나눔수 <- (12) 곱수.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_mismatch_exact_numeric.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("E_RUNTIME_TYPE_MISMATCH"),
            "unexpected error: {message}"
        );
        assert!(message.contains("핀=값"), "unexpected error: {message}");
        assert!(
            message.contains("기대=나눔수"),
            "unexpected error: {message}"
        );
        assert!(message.contains("실제=곱수"), "unexpected error: {message}");
    }

    #[test]
    fn decl_block_type_alias_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:rational <- (12) 곱수.
    }.
}
"#;
        let program = DdnProgram::from_source(script, "decl_type_alias_mismatch_rational.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("E_RUNTIME_TYPE_MISMATCH"),
            "unexpected error: {message}"
        );
        assert!(
            message.contains("기대=나눔수"),
            "unexpected error: {message}"
        );
        assert!(
            !message.contains("기대=rational"),
            "expected canonical type name, got: {message}"
        );
    }

    #[test]
    fn decl_block_type_alias_accepts_collection_and_string_none_aliases() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        글값:string <- "또니".
        없음값:none <- 없음.
        비움값:non <- 없음.
        목록값:목록 <- (1, 2) 차림.
        모둠값:모둠 <- (1, 2, 2) 모음.
        그림표값:그림표 <- ("x", 1) 짝맞춤.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_accepts_collection_string_none.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run should succeed");
    }

    #[test]
    fn decl_block_type_alias_none_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:none <- 1.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_none_mismatch.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("기대=없음"),
            "expected canonical type name, got: {message}"
        );
        assert!(
            !message.contains("기대=none"),
            "alias name should not leak in canonical mismatch message: {message}"
        );
    }

    #[test]
    fn decl_block_type_alias_non_keyword_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:non <- 1.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_non_mismatch.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("기대=없음"),
            "expected canonical type name, got: {message}"
        );
        assert!(
            !message.contains("기대=non"),
            "alias name should not leak in canonical mismatch message: {message}"
        );
    }

    #[test]
    fn decl_block_infer_type_skips_runtime_typecheck() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:_ <- (12) 곱수.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_infer_skips_typecheck.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run should succeed");
    }

    #[test]
    fn numeric_kernel_factor_complete_large_power_shared_crate() {
        let input: BigInt = (BigInt::from(1u8) << 520) * BigInt::from(9u8);
        let outcome =
            ddonirang_numeric::complete_factor(&input.to_string()).expect("numeric kernel factor");
        assert_eq!(outcome.result.status, "done");
        assert_eq!(outcome.result.canonical.as_deref(), Some("2^520 * 3^2"));
        assert!(outcome.result.certificate.product_matches_input);
    }

    #[test]
    fn parse_warnings_include_block_header_colon_deprecation() {
        let script = r#"
매틱:움직씨 = {
    채비: { 값:수 <- 0. }.
}
"#;
        let program = DdnProgram::from_source(script, "warn_block_header.ddn").expect("parse");
        assert!(program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn parse_warnings_include_chaebi_reassign_warning() {
        let script = r#"
매틱:움직씨 = {
    채비 { 값:수 <- 0. }.
    값 <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "warn_chaebi_reassign.ddn").expect("parse");
        assert!(program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn parse_warnings_skip_chaebi_derived_reassign_warning() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        점수:수 <- 72.
        통과함:참거짓 <- 거짓.
    }.
    통과함 <- 점수 >= 70.
    통과함 보여주기.
}
"#;
        let program = DdnProgram::from_source(script, "warn_chaebi_derived.ddn").expect("parse");
        assert!(!program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn top_level_chaebi_assignment_updates_resource_and_show_log() {
        let script = r#"
채비 {
  점수: 수 <- 72.
  통과함: 참거짓 <- 거짓.
}.

통과함 <- 점수 >= 70.

통과함 보여주기.
"#;
        let program = DdnProgram::from_source(script, "top_level_show.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        match output.resources.get("통과함") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("통과함 must be true, got {:?}", other),
        }
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("output_log must be list");
        };
        let first = entries.first().expect("first output log entry");
        let RuntimeValue::Map(map) = first else {
            panic!("output log entry must be map");
        };
        let text = map
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|entry| match &entry.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("참"));
    }

    #[test]
    fn public_observation_emits_ordered_committed_rust_events_without_state_or_graph_inference() {
        let script = r#"
채비 {
  시간 <- 0.
  각도 <- 15.
  각속도 <- 2.
}.

(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 각속도.
  (시간: 시간 * 1@s, 각도: 각도 * 1@rad, 각속도: 각속도) 내보임.
}.
"#;
        let program = DdnProgram::from_source(script, "관찰/진자.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매마디");
        let mut input = empty_input();
        input.tick_id = 7;
        let output = runner
            .run_update(&NuriWorld::new(), &input, &HashMap::new())
            .expect("run update");

        assert_eq!(
            output
                .public_observations
                .iter()
                .map(|event| event.name.as_str())
                .collect::<Vec<_>>(),
            vec!["시간", "각도", "각속도"]
        );
        assert!(output
            .public_observations
            .iter()
            .all(|event| event.committed_madi == 7));
        assert_eq!(
            output
                .public_observations
                .iter()
                .map(|event| event.unit.as_str())
                .collect::<Vec<_>>(),
            vec!["s", "rad", ""]
        );
        assert_eq!(
            output
                .public_observations
                .iter()
                .map(|event| event.value_text.as_str())
                .collect::<Vec<_>>(),
            vec!["1@s", "17@rad", "2"]
        );
        assert_eq!(
            output
                .public_observations
                .iter()
                .map(|event| event.starts_observation_event)
                .collect::<Vec<_>>(),
            vec![true, false, false]
        );
        for event in &output.public_observations {
            assert!(event.source_ref.starts_with("관찰/진자.ddn#bytes="));
            assert_eq!(
                &script[event.source_start..event.source_end],
                event.name.as_str()
            );
            assert_eq!(event.format, "ddn-canonical");
            assert!(!event.source_ref.contains("project.obs."));
            assert!(!event.source_ref.contains("output_"));
        }
        assert!(!output.resources.contains_key("__view_public_observations"));
        assert!(!output.resources.contains_key("보개_그래프_점목록_f"));
        assert!(!output.resources.contains_key("보개_출력_줄들"));
    }

    fn sealed_dataset_registry_fixture() -> Arc<gate0_registry::ProjectAssetRegistry> {
        use crate::sealed_domain_dataset::{
            canonical_sealed_domain_dataset_json, seal_domain_dataset, DatasetColumn, DatasetDtype,
            DatasetPayload, DatasetRow, DatasetScalar, DatasetSource, SealedDomainDataset,
            SEALED_DOMAIN_DATASET_SCHEMA,
        };

        let payload = b"month,expense\n1,700\n2,\n".to_vec();
        let dataset = seal_domain_dataset(
            SealedDomainDataset {
                schema: SEALED_DOMAIN_DATASET_SCHEMA.to_string(),
                artifact_id: "fixture.runtime-open".to_string(),
                artifact_version: "v1".to_string(),
                source: DatasetSource::ExternalSnapshot {
                    provider: "W1 fixture provider".to_string(),
                    title: "runtime open fixture".to_string(),
                    canonical_uri: "https://example.invalid/ddn/runtime-open".to_string(),
                    published_or_version: "fixture-v1".to_string(),
                    retrieved_at: "2026-07-27".to_string(),
                    license_id: "LicenseRef-W1-fixture".to_string(),
                    attribution: "W1 runtime fixture".to_string(),
                },
                payload: DatasetPayload {
                    object_ref: "datasets/runtime-open/payload.csv".to_string(),
                    media_type: "text/csv".to_string(),
                    byte_length: 0,
                    sha256: String::new(),
                },
                ordered_columns: vec![
                    DatasetColumn {
                        name: "월".to_string(),
                        dtype: DatasetDtype::String,
                        unit: None,
                        source_locator: "csv:column:1".to_string(),
                    },
                    DatasetColumn {
                        name: "지출".to_string(),
                        dtype: DatasetDtype::Integer,
                        unit: Some("원".to_string()),
                        source_locator: "csv:column:2".to_string(),
                    },
                ],
                ordered_rows: vec![
                    DatasetRow {
                        source_ordinal: 0,
                        cells: vec![
                            DatasetCell::Present {
                                value: DatasetScalar::String {
                                    value: "1월".to_string(),
                                },
                            },
                            DatasetCell::Present {
                                value: DatasetScalar::Integer { value: 700 },
                            },
                        ],
                    },
                    DatasetRow {
                        source_ordinal: 1,
                        cells: vec![
                            DatasetCell::Present {
                                value: DatasetScalar::String {
                                    value: "2월".to_string(),
                                },
                            },
                            DatasetCell::Missing {
                                reason: MissingReason::SourceUnavailable,
                            },
                        ],
                    },
                ],
                content_sha256: String::new(),
                schema_sha256: String::new(),
            },
            &payload,
        )
        .expect("seal runtime fixture");
        let envelope = canonical_sealed_domain_dataset_json(&dataset).expect("canonical envelope");
        let bundle_id = "ddn.w1.runtime-open";
        let envelope_path = "datasets/runtime-open/envelope.json";
        let payload_path = "datasets/runtime-open/payload.csv";
        let envelope_handle =
            ddonirang_core::asset_handle_from_bundle_path(bundle_id, envelope_path).raw();
        let payload_handle =
            ddonirang_core::asset_handle_from_bundle_path(bundle_id, payload_path).raw();
        let manifest = serde_json::to_vec(&serde_json::json!({
            "version": "v0",
            "bundle_id": bundle_id,
            "hash_algo": "blake3",
            "entries": [
                {
                    "path": envelope_path,
                    "handle": format!("0x{envelope_handle:016x}"),
                    "hash": blake3::hash(envelope.as_bytes()).to_hex().to_string(),
                    "size": envelope.len(),
                    "mime": "application/vnd.ddn.sealed-domain-dataset+json"
                },
                {
                    "path": payload_path,
                    "handle": format!("0x{payload_handle:016x}"),
                    "hash": blake3::hash(&payload).to_hex().to_string(),
                    "size": payload.len(),
                    "mime": "text/csv"
                }
            ]
        }))
        .expect("manifest");
        let files = HashMap::from([
            (envelope_path.to_string(), envelope.into_bytes()),
            (payload_path.to_string(), payload),
        ]);
        Arc::new(
            gate0_registry::build_project_asset_registry_from_manifest(&manifest, &files, true)
                .expect("asset registry"),
        )
    }

    #[test]
    fn dataset_open_uses_verified_handle_and_preserves_ordered_cells_in_product_runtime() {
        let script = r#"
매틱:움직씨 = {
  자료값 <- (@"datasets/runtime-open/envelope.json") 자료.열기.
  첫행 <- (자료값.행, 0) 차림.값.
  월셀 <- (첫행.cells, 0) 차림.값.
  지출셀 <- (첫행.cells, 1) 차림.값.
  (월: 월셀.value, 지출: 지출셀.value) 내보임.
}
"#;
        let program = DdnProgram::from_source(script, "dataset_open.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱")
            .with_project_asset_registry(sealed_dataset_registry_fixture());
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("dataset open runtime");
        assert_eq!(output.public_observations.len(), 2);
        assert_eq!(output.public_observations[0].name, "월");
        assert_eq!(output.public_observations[0].dtype, "string");
        assert_eq!(output.public_observations[0].unit, "");
        assert_eq!(output.public_observations[0].value_text, "1월");
        assert_eq!(
            output.public_observations[0].dataset_target,
            Some(PublicObservationDatasetTarget {
                dataset_content_sha256:
                    "0e2ddf25d6a41f22a561b93232faa63e2048059ed4e2c0ac3d91387236badbdd".to_string(),
                row_source_ordinal: 0,
                column_name: "월".to_string(),
            })
        );
        assert_eq!(output.public_observations[1].name, "지출");
        assert_eq!(output.public_observations[1].dtype, "integer");
        assert_eq!(output.public_observations[1].unit, "원");
        assert_eq!(output.public_observations[1].value_canon, "700");

        let unverified_script = r#"
매틱:움직씨 = {
  자료값 <- (가짜) 자료.열기.
}
"#;
        let program = DdnProgram::from_source(unverified_script, "dataset_open_unverified.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱")
            .with_project_asset_registry(sealed_dataset_registry_fixture());
        let defaults = HashMap::from([(
            "가짜".to_string(),
            RuntimeValue::ResourceHandle(ResourceHandle::from_path(
                "datasets/runtime-open/envelope.json",
            )),
        )]);
        let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &defaults) {
            Ok(_) => panic!("unverified handle must fail"),
            Err(error) => error,
        };
        assert!(error.contains("E_DATASET_HANDLE_UNVERIFIED"), "{error}");
    }

    #[test]
    fn dataset_scalar_mol_is_numeric_only_for_exact_mol_metadata() {
        let mol_column = DatasetColumn {
            name: "초기 물질량".to_string(),
            dtype: DatasetDtype::Fixed64,
            unit: Some("mol".to_string()),
            source_locator: "csv:column:5".to_string(),
        };
        let mol = dataset_scalar_runtime_value(
            &DatasetScalar::Fixed64 {
                value: "2".to_string(),
            },
            &mol_column,
            &"a".repeat(64),
            0,
        );
        assert_eq!(
            unit_value_from_value(&mol).expect("exact mol dataset scalar"),
            UnitValue {
                value: Fixed64::from_i64(2),
                dim: UnitDim::AMOUNT_OF_SUBSTANCE,
            }
        );

        let coefficient_column = DatasetColumn {
            name: "계수".to_string(),
            dtype: DatasetDtype::Integer,
            unit: None,
            source_locator: "csv:column:4".to_string(),
        };
        let coefficient = dataset_scalar_runtime_value(
            &DatasetScalar::Integer { value: 2 },
            &coefficient_column,
            &"b".repeat(64),
            0,
        );
        assert_eq!(
            unit_value_from_value(&coefficient).expect("dimensionless coefficient"),
            UnitValue {
                value: Fixed64::from_i64(2),
                dim: UnitDim::NONE,
            }
        );

        for tampered_unit in ["kg", "mol/L", "MOL", ""] {
            let mut tampered = mol_column.clone();
            tampered.unit = Some(tampered_unit.to_string());
            let value = dataset_scalar_runtime_value(
                &DatasetScalar::Fixed64 {
                    value: "2".to_string(),
                },
                &tampered,
                &"c".repeat(64),
                0,
            );
            assert!(
                unit_value_from_value(&value).is_err(),
                "unexpected dataset unit became numeric: {tampered_unit}"
            );
        }
    }

    #[test]
    fn standard_observation_limit_failure_keeps_candidate_state_uncommitted() {
        let source = r#"
매틱:움직씨 = {
  임시 <- 7.
  (첫째: 임시, 둘째: 2) 내보임.
}
"#;
        let program = DdnProgram::from_source(source, "work_cost_atomic.ddn").expect("parse");
        let limits = StandardResourceLimitsV1 {
            public_observation_samples: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        };
        let mut runner = DdnRunner::new(program, "매틱").with_standard_resource_limits(limits);
        let world = NuriWorld::new();
        let error = match runner.run_update_diagnostic(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("two samples must exceed the one-sample ceiling"),
            Err(error) => error,
        };
        let (code, message, _) = error.diagnostic_parts();
        assert_eq!(
            code,
            crate::work_cost::E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE
        );
        assert!(message.contains("public_observation_samples"), "{message}");
        assert!(message.contains("work_cost_receipt="), "{message}");
        assert_eq!(
            runner
                .work_cost_receipt()
                .counters
                .public_observation_samples,
            0
        );
        assert_eq!(runner.work_cost_receipt().counters.committed_madi, 0);
        assert!(world.get_resource_value("임시").is_none());
    }

    #[test]
    fn work_cost_exact_numeric_charges_constructor_and_infix_operand_result_bytes() {
        let source = r#"
매틱:움직씨 = {
  a <- ("1000") 큰바른수.
  b <- ("1") 큰바른수.
  c <- a + b.
}
"#;
        let program =
            DdnProgram::from_source(source, "work_cost_exact_numeric.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("run update");

        // 큰바른수(1000): operand 3 + result 3, 큰바른수(1): 2 + 2,
        // exact infix: operands 3 + 2, result 3.
        assert_eq!(
            output
                .work_cost
                .counters
                .bigint_rational_signed_magnitude_byte_visits,
            18
        );
        assert_eq!(output.work_cost.counters.call_depth_current, 0);
        assert_eq!(output.work_cost.counters.call_depth_peak, 1);
    }

    #[test]
    fn callback_error_charges_only_executed_collection_prefix_and_unwinds_depth() {
        let source = r#"
매틱:움직씨 = {
  목록 <- (1, 0, 3) 차림.
  결과 <- (목록, {값| 없는값}) 변환.
}
"#;
        let program =
            DdnProgram::from_source(source, "work_cost_callback_prefix.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let error = match runner.run_update_diagnostic(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("first callback must fail"),
            Err(error) => error,
        };
        assert!(error.message().contains("없는값"), "{error:?}");
        let receipt = runner.work_cost_receipt();
        // 기존 연산: 생성 3 + 첫 callback 1. 뒤 callback은 실행되지 않는다.
        // 관계 보호 검사는 먼저 차림 자체와 3요소, callback 값까지 방문한다.
        let relation_guard_visits = 1 + 3 + 1;
        assert_eq!(receipt.counters.collection_tensor_element_visits, 4 + relation_guard_visits);
        assert_eq!(receipt.counters.call_depth_current, 0);
        assert_eq!(receipt.counters.call_depth_peak, 2);
        assert_eq!(receipt.counters.committed_madi, 0);
        assert!(world.get_resource_value("결과").is_none());
    }

    #[test]
    fn collection_search_and_point_access_charge_only_actual_visits() {
        let source = r#"
매틱:움직씨 = {
  목록 <- (10, 20, 30) 차림.
  첫값 <- (목록) 첫번째.
  있나 <- (목록, 20) 들어있나.
  위치 <- (목록, 99) 찾아보기.
  조각 <- (목록, 1, 3) 토막내기.
}
"#;
        let program =
            DdnProgram::from_source(source, "work_cost_collection_visits.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
            .expect("run update");

        // 기존 연산: 생성 3 + 첫번째 1 + 검색 2 + 없음 검색 3 + slice 복사 2.
        // 각 호출의 관계 보호: 목록(1+3), 추가 scalar 받는자리(0/1/1/2).
        let relation_guard_visits = 4 + 5 + 5 + 6;
        assert_eq!(
            output.work_cost.counters.collection_tensor_element_visits,
            11 + relation_guard_visits
        );
    }

    #[test]
    fn join_type_error_charges_only_validated_prefix() {
        let source = r#"
매틱:움직씨 = {
  목록 <- ("가", "b", 3) 차림.
  결과 <- (목록, ",") 합치기.
}
"#;
        let program = DdnProgram::from_source(source, "work_cost_join_prefix.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("third join item must fail"),
            Err(error) => error,
        };
        assert!(error.message().contains("차림<글>"), "{error:?}");
        let receipt = runner.work_cost_receipt();
        // 기존 연산: 생성 3 + join 방문 3. 관계 보호: 목록(1+3) + 구분 글 1.
        let relation_guard_visits = 4 + 1;
        assert_eq!(receipt.counters.collection_tensor_element_visits, 6 + relation_guard_visits);
        // literal 생성 5 bytes + join prefix의 "가" 3, separator 1, "b" 1.
        // 실패한 셋째 앞에는 separator를 쓰지 않는다.
        assert_eq!(receipt.counters.canonical_utf8_byte_visits, 10);
        assert_eq!(receipt.counters.committed_madi, 0);
    }

    #[test]
    fn string_conversion_follow_up_error_preserves_executed_utf8_visit() {
        let source = r#"
매틱:움직씨 = {
  결과 <- ("abc") 숫자로.
}
"#;
        let program =
            DdnProgram::from_source(source, "work_cost_string_error_prefix.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("none assignment after invalid numeric text must fail"),
            Err(error) => error,
        };
        assert!(error.message().contains("대입"), "{error:?}");
        let receipt = runner.work_cost_receipt();
        // literal construction 3 bytes + 숫자로 conversion visit 3 bytes.
        assert_eq!(receipt.counters.canonical_utf8_byte_visits, 6);
        assert_eq!(receipt.counters.committed_madi, 0);
    }

    #[test]
    fn aggregate_type_error_charges_only_executed_collection_prefix() {
        let source = r#"
매틱:움직씨 = {
  목록 <- (1, "아님", 3) 차림.
  결과 <- (목록) 합계.
}
"#;
        let program =
            DdnProgram::from_source(source, "work_cost_aggregate_error_prefix.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let error = match runner.run_update_diagnostic(
            &NuriWorld::new(),
            &empty_input(),
            &HashMap::new(),
        ) {
            Ok(_) => panic!("second aggregate element must fail"),
            Err(error) => error,
        };
        assert!(
            error.message().contains("수") || error.message().contains("타입"),
            "{error:?}"
        );
        let receipt = runner.work_cost_receipt();
        // 기존 연산: 생성 3 + 합계 prefix 2. 관계 보호는 목록(1+3)을 먼저 검사한다.
        let relation_guard_visits = 1 + 3;
        assert_eq!(receipt.counters.collection_tensor_element_visits, 5 + relation_guard_visits);
        assert_eq!(receipt.counters.committed_madi, 0);
    }

    #[test]
    fn callback_call_depth_boundary_plus_one_is_atomic() {
        let source = r#"
매틱:움직씨 = {
  목록 <- (1) 차림.
  결과 <- (목록, {값| 값}) 변환.
}
"#;
        let program = DdnProgram::from_source(source, "work_cost_call_depth.ddn").expect("parse");
        let limits = StandardResourceLimitsV1 {
            call_depth: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        };
        let mut runner = DdnRunner::new(program, "매틱").with_standard_resource_limits(limits);
        let world = NuriWorld::new();
        let error = match runner.run_update_diagnostic(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("callback depth two must exceed limit one"),
            Err(error) => error,
        };
        assert_eq!(
            error.diagnostic_parts().0,
            crate::work_cost::E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE
        );
        let receipt = runner.work_cost_receipt();
        assert_eq!(receipt.counters.call_depth_current, 0);
        assert_eq!(receipt.counters.call_depth_peak, 1);
        assert_eq!(receipt.counters.committed_madi, 0);
        assert!(world.get_resource_value("결과").is_none());
    }

    #[test]
    fn created_identity_boundary_plus_one_is_atomic() {
        let source = r#"
매틱:움직씨 = {
  첫째 <- {값| 값}.
  둘째 <- {값| 값}.
}
"#;
        let program = DdnProgram::from_source(source, "work_cost_identity.ddn").expect("parse");
        let limits = StandardResourceLimitsV1 {
            created_identities: 1,
            ..StandardResourceLimitsV1::CANDIDATE
        };
        let mut runner = DdnRunner::new(program, "매틱").with_standard_resource_limits(limits);
        let world = NuriWorld::new();
        let error = match runner.run_update_diagnostic(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("second lambda identity must exceed limit one"),
            Err(error) => error,
        };
        assert_eq!(
            error.diagnostic_parts().0,
            crate::work_cost::E_STANDARD_RESOURCE_LIMIT_EXCEEDED_CANDIDATE
        );
        let receipt = runner.work_cost_receipt();
        assert_eq!(receipt.counters.created_identities, 1);
        assert_eq!(receipt.counters.call_depth_current, 0);
        assert_eq!(receipt.counters.committed_madi, 0);
        assert!(world.get_resource_value("첫째").is_none());
        assert!(world.get_resource_value("둘째").is_none());
    }
