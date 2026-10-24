//! Existing tests; namespace and included source bytes preserved.
    #[test]
    fn wasm_run_summary_export_preserves_native_compact_response_bytes() {
        let source = "(매마디)마다 { 값 <- 차림[1, 2]. }.";
        let native = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "compact-parity.ddn", Some(1), Some("v1-core-v25")).unwrap();
        let exported = super::wasm_run_summary_with_source_profile_json(
            source, "compact-parity.ddn", 1, "v1-core-v25").unwrap();
        assert_eq!(exported, native.to_string());
        assert!(!exported.ends_with('\n'));
    }

    use super::*;
    #[test]
    fn wasm_current_state_response_copies_deep_history_without_recursion() {
        std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
            use crate::unit_wire::SnapshotJsonOwner as Owned;
            let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
            let shallow = crate::unit_wire::parse_snapshot_json(&vm.current_state_json_text()).unwrap();
            assert_eq!(shallow["public_observations"], shallow["state"]["public_observations"]);
            assert_eq!(shallow["debug_pause"], shallow["state"]["debug_pause"]);
            assert!(shallow["output_rows"].as_array().is_some());
            // Independent legacy shallow serialization oracle: exact bytes,
            // including the deprecated columns shape and absence of new keys.
            let expected_columns = json!({
                "observation_columns": vm.public_observation_trace.get("columns").cloned().unwrap_or_else(|| json!([])),
                "output_rows": vm.public_observation_trace.get("samples").cloned().unwrap_or_else(|| json!([])),
                "public_observation_trace": vm.public_observation_trace,
                "public_observation_trace_hash": vm.public_observation_trace_hash,
                "public_observation_table_projection": vm.public_observation_table_projection,
                "deprecated": true,
                "use_instead": "step_one() 또는 get_state_json()의 public_observation_trace",
            }).to_string();
            assert_eq!(vm.columns_json_text(), expected_columns);
            let mut value = JsonValue::Bool(true);
            for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
            let deep = Owned::from_value(value);
            vm.last_public_observations.push(deep.clone().into_parent_value());
            vm.public_observation_frames.push(JsonValue::Object([
                ("committed_madi".into(), 0.into()),
                ("checkpoint".into(), deep.clone().into_parent_value()),
            ].into_iter().collect()));
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_trace,
                JsonValue::Object([("columns".into(), deep.clone().into_parent_value()),
                    ("samples".into(), deep.clone().into_parent_value())].into_iter().collect()));
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_table_projection, deep.clone().into_parent_value());
            crate::unit_wire::replace_retained_json(&mut vm.last_debug_evidence, deep.clone().into_parent_value());
            let before = vm.get_state_hash();
            let encoded = vm.current_state_json_text();
            // The transitional restore depth gate remains enabled. Verify
            // export bytes without trying to admit this synthetic deep state.
            let exact_deep = format!("{}true{}", "[".repeat(20_000), "]".repeat(20_000));
            for key in ["observation_columns", "output_rows", "public_observation_table_projection", "debug_pause"] {
                assert!(encoded.contains(&format!("\"{key}\":{exact_deep}")), "{key}");
            }
            assert_eq!(encoded.matches(&format!("\"public_observations\":[{exact_deep}]")).count(), 2);
            assert!(encoded.contains(&format!("\"checkpoint\":{exact_deep}")));
            assert!(crate::unit_wire::parse_snapshot_json(&encoded).is_err());
            let columns = vm.columns_json_text();
            for key in ["observation_columns", "output_rows", "public_observation_table_projection"] {
                assert!(columns.contains(&format!("\"{key}\":{exact_deep}")));
            }
            assert!(!columns.contains("\"public_observations\":"));
            assert!(!columns.contains("\"debug_pause\":"));
            assert_eq!(vm.get_state_hash(), before);
            // Ownership/response safety only: these synthetic nested fields
            // are not an admission or replay claim for a new observation type.
        }).unwrap().join().unwrap();
    }

    #[test]
    fn wasm_retained_observation_json_replacement_reset_and_close_are_stack_safe() {
        // VM owns thread-local runtime state; do not move it between threads.
        std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
            let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
            let deep = || {
                let mut value = JsonValue::Null;
                for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
                value
            };
            vm.last_bogae_projection = Some(deep());
            vm.last_public_observations.push(deep());
            vm.public_observation_frames.push(deep());
            vm.public_observation_trace = deep();
            vm.public_observation_table_projection = deep();
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_trace, deep());
            reset_wasm_public_observation_surface(&mut vm).unwrap();
            assert!(vm.last_bogae_projection.is_none());
            assert!(vm.public_observation_frames.is_empty());
            assert!(vm.last_public_observations.is_empty());
            vm.public_observation_frames.push(deep());
            vm.last_debug_evidence = deep();
            drop(vm);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn wasm_snapshot_builder_transfers_deep_resources_without_reencoding() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = JsonValue::Bool(true);
            for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
            let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(build_wasm_state_snapshot_v2(
                value, json!({}), vec![], 7, json!({})));
            assert_eq!(snapshot["next_tick_id"], json!(7));
            let copied = snapshot.clone();
            assert!(copied == snapshot);
            assert_eq!(copied.compact_text(), snapshot.compact_text());
        }).unwrap().join().unwrap();
        let resources = json!({"tree":[null,1.25,"한글"]});
        let expected = json!({"schema":WASM_STATE_SNAPSHOT_SCHEMA_V2,
            "resource_snapshot_schema":RESOURCE_SNAPSHOT_SCHEMA_V2,"unit_wire_schema":UNIT_VALUE_WIRE_SCHEMA_V2,
            "resources":resources.clone(),"param_overrides":{},"public_observation_frames":[],
            "next_tick_id":7,"scene_history":{}});
        let actual = build_wasm_state_snapshot_v2(resources, json!({}), vec![], 7, json!({}));
        assert_eq!(actual.to_string(), expected.to_string());
    }
    use ddonirang_core::{
        replay_sealed_external_execution, CanonicalExternalInput, CapabilityBinding, CapabilityRef,
        CapabilityScope, CapabilityValue, DependencyNodeKind, DeterministicSealedProvider,
        ExternalBoundaryRequest, ExternalDependencyNode,
    };
    use ddonirang_core::platform::ResourceMapEntry;
    use sha2::{Digest, Sha256};

    #[test]
    fn d9_wasm_vm_binds_named_project_entry_after_profile_update() {
        let source = "중력:수 := 9.8. 실행:움직씨 := { 중력 보여주기. }";
        let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
        vm.update_logic_with_source_profile(source, "v1-core-v25").unwrap();
        vm.configure_project_entry("실행").unwrap();
        let input = InputSnapshot {
            tick_id: 0, dt: Fixed64::from_i64(1), keys_pressed: 0,
            last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
            ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: InputSource::Person, rng_seed: 0,
        };
        let output = vm.runner.run_update_with_param_overrides(vm.world.world(), &input,
            &vm.defaults, &vm.param_overrides).unwrap();
        assert!(output.resources.contains_key("중력"));
    }

    #[test]
    fn wasm_vm_profile_update_replaces_inert_bootstrap_with_v25_source() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let mut vm = DdnWasmVm::new("1 보여주기.").expect("inert bootstrap VM");
        vm.update_logic_with_source_profile(source, "v1-core-v25")
            .expect("V25 profile update");
        assert_eq!(vm.get_configured_madi(), 0);
        assert_eq!(vm.source_bytes_sha256, runtime_surface::sha256_hex(source.as_bytes()));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wo02_invalid_vm_update_preserves_installed_source() {
        let valid = "값 <- 1. 값 보여주기.";
        let invalid = "값 <- 1. 값 <- \"가\".";
        assert!(DdnWasmVm::new(invalid).is_err());
        let mut vm = DdnWasmVm::new(valid).expect("valid source");
        let before = vm.source_bytes_sha256.clone();
        assert!(vm.update_logic(invalid).is_err());
        assert_eq!(vm.source_bytes_sha256, before);
        assert!(vm.update_logic_with_mode(invalid, "strict").is_err());
        assert_eq!(vm.source_bytes_sha256, before);
        assert!(vm.update_logic_with_source_profile(invalid, "v1-core-v25").is_err());
        assert_eq!(vm.source_bytes_sha256, before);
    }

    #[test]
    fn wasm_fixed64_input_v1_reconstructs_q31_32_without_f64_conversion() {
        let mut vm = DdnWasmVm::new("1 보여주기.").expect("inert bootstrap VM");

        vm.set_input_fixed64_parts(7, "k", -3, 9, i32::MIN, 0);
        assert_eq!(vm.input_dt, Fixed64::MIN);
        assert_eq!(vm.input_keys_pressed, 7);
        assert_eq!(vm.input_last_key_name, "k");
        assert_eq!(vm.input_pointer_x_i32, -3);
        assert_eq!(vm.input_pointer_y_i32, 9);

        vm.set_dt_fixed64_parts(0, 1);
        assert_eq!(vm.input_dt, Fixed64::from_raw_i64(1));
        assert_eq!(
            fixed64_q31_32_parts(Fixed64::MIN),
            json!({"high_i32": i32::MIN, "low_u32": 0})
        );
        for value in [
            Fixed64::MIN,
            Fixed64::MAX,
            Fixed64::from_raw_i64(-1),
            Fixed64::from_raw_i64(1),
            Fixed64::from_raw_i64(0x0000_0000_FFFF_FFFF),
        ] {
            let parts = fixed64_q31_32_parts(value);
            let high = parts["high_i32"].as_i64().expect("high") as i32;
            let low = parts["low_u32"].as_u64().expect("low") as u32;
            assert_eq!(fixed64_from_q31_32_parts(high, low), value);
        }
        assert_eq!(
            Fixed64::parse_decimal_exact("0.5"),
            Some(fixed64_from_q31_32_parts(0, 0x8000_0000))
        );
    }


    fn sealed_external_test_request() -> (ProcessLocalCapabilityStore, ExternalBoundaryRequest) {
        let binding = CapabilityBinding::new("owner-a", "world-a", 7, 3);
        let mut store = ProcessLocalCapabilityStore::new(7);
        assert!(store.insert(CapabilityValue::new(
            CapabilityRef::from_process_local(41),
            binding.clone(),
            [CapabilityScope::ExternalBoundaryExecute { boundary_id: "boundary:one".to_string() }],
        )));
        let request = ExternalBoundaryRequest {
            dependency: ExternalDependencyNode {
                kind: DependencyNodeKind::ExternalArtifact,
                artifact_identity: "artifact:sealed-01".to_string(),
                provenance_identity: "provenance:sealed-01".to_string(),
                provider_owner: "provider:sealed".to_string(),
                callable_owner: "callable:transform".to_string(),
                invocation_identity: "invocation:one".to_string(),
                execution_contract: "contract:sealed-v1".to_string(),
                external_boundary_identity: "boundary:one".to_string(),
            },
            input: CanonicalExternalInput { canonical_text: "입력:42".to_string() },
            capability: Some(CapabilityRef::from_process_local(41)),
            capability_binding: binding,
            max_input_bytes: 64,
            max_output_bytes: 64,
            max_calls: 1,
            call_ordinal: 1,
        };
        (store, request)
    }

    #[test]
    fn external_boundary_native_and_wasm_adapters_share_one_typed_owner() {
        let provider = DeterministicSealedProvider::new("provider:sealed", "callable:transform");
        let (native_store, request) = sealed_external_test_request();
        let (wasm_store, wasm_request) = sealed_external_test_request();
        let mut native_ledger = SealedExecutionLedger::new();
        let mut wasm_ledger = SealedExecutionLedger::new();

        let native = runtime_surface::execute_sealed_external_boundary(
            &mut native_ledger, &native_store, &provider, &request,
        ).expect("native adapter");
        let wasm = wasm_execute_sealed_external_boundary(
            &mut wasm_ledger, &wasm_store, &provider, &wasm_request,
        ).expect("wasm adapter");

        assert_eq!(native, wasm);
        assert_eq!(native.dispatch_count, 1);
        assert_eq!(native.publication_count, 1);
        assert_eq!(
            replay_sealed_external_execution(
                &request.dependency, &request.input, &native.output, &native.receipt,
            ),
            Ok(())
        );
    }

    #[test]
    fn declared_host_resource_native_and_wasm_share_one_receipt_bound_payload() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
        let text = "고정 입력\n";
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let request = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "record",
            "resources": [{
                "logical_resource_id": "caption",
                "sha256": format!("{:x}", hasher.finalize()),
                "byte_length": text.len(),
                "media_type": crate::host_resource::V1_TEXT_MEDIA_TYPE,
                "utf8_text": text,
            }]
        })
        .to_string();
        let parsed = crate::host_resource::parse_request_json(&request).expect("typed request");
        let prepared = crate::host_resource::prepare_request(&parsed).expect("record preflight");
        let native = runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
            source,
            "host-resource.ddn",
            Some(1),
            Some("v1-core-v25"),
            &prepared,
        )
        .expect("native shared runtime");
        let wasm = wasm_run_summary_with_source_profile_host_resources_json(
            source,
            "host-resource.ddn",
            1,
            "v1-core-v25",
            &request,
        )
        .expect("WASM shared runtime");
        assert_eq!(wasm, native.to_string());
        assert_eq!(
            native["host_resource_receipt"]["receipt_identity"],
            prepared.receipt.receipt_identity
        );

        let replay = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "replay",
            "transcript": prepared.transcript,
        })
        .to_string();
        assert_eq!(
            wasm_run_summary_with_source_profile_host_resources_json(
                source,
                "host-resource.ddn",
                1,
                "v1-core-v25",
                &replay,
            )
            .expect("WASM replay"),
            native.to_string()
        );
    }

    #[test]
    fn p1a_supported_profile_native_wasm_payloads_are_exactly_equal() {
        let cases = [
            (
                "p1a-parity.ddn",
                "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n",
            ),
            (
                "vc12-collection-parity.ddn",
                "매마디:움직씨 := {\n    값들:차림 := (1, 2, 3) 차림.\n    값들 보여주기.\n}\n",
            ),
            (
                "pg-cli-result-flow-parity.ddn",
                "매마디:움직씨 := {\n    판:(수)차림 := 차림[0, 0].\n    1 -> 판[0].\n    판 보여주기.\n}\n",
            ),
        ];
        for (path, source) in cases {
            let native = runtime_surface::run_summary_from_supported_source_profile(
                source,
                path,
                Some(1),
                Some("v1-core-v25"),
            )
            .expect("native supported profile summary")
            .to_string();
            let wasm = wasm_run_summary_with_source_profile_json(
                source,
                path,
                1,
                "v1-core-v25",
            )
            .expect("WASM supported profile summary");
            assert_eq!(wasm, native, "path={path}");
        }

        let source = cases[0].1;

        let native_canon = runtime_surface::canonicalize_shared_source_with_supported_profile(
            source,
            Some("v1-core-v25"),
        )
        .expect("native supported profile canon")
        .ddn;
        let wasm_canon = wasm_canon_ddn_with_source_profile(source, "v1-core-v25")
            .expect("WASM supported profile canon");
        assert_eq!(wasm_canon, native_canon);

        let request = serde_json::json!({
            "source_text": source,
            "path": "p1a-parity.ddn",
            "analysis_generation": 201,
            "source_profile_identity": "v1-core-v25"
        })
        .to_string();
        assert_eq!(
            wasm_source_analysis_snapshot_json(&request).expect("WASM source analysis"),
            source_analysis::source_analysis_snapshot_json(&request)
                .expect("native source analysis")
        );

        let module_request = serde_json::json!({
            "schema": module_directive_analysis::MODULE_DIRECTIVE_ANALYSIS_SCHEMA,
            "source_text": "(운동=\"./lib/😀\") 쓰임.\n(속도=중력) 드러냄.\n계산:움직씨 := {\n  값:수 := 1.\n}\n",
            "path": "운동😀.ddn",
            "analysis_generation": 202,
            "source_profile_identity": "v1-core-v25",
            "module_registry": {
                "known_import_targets": ["./lib/😀"],
                "public_definition_targets": ["중력"]
            }
        })
        .to_string();
        assert_eq!(
            wasm_module_directive_analysis_snapshot_json(&module_request)
                .expect("WASM module directive analysis"),
            module_directive_analysis::module_directive_analysis_snapshot_json(&module_request)
                .expect("native module directive analysis")
        );
    }

    #[test]
    fn committed_madi_is_last_successfully_committed_frame_not_next_tick() {
        assert_eq!(last_committed_madi(&[]), JsonValue::Null);
        let frames = vec![
            json!({"committed_madi": 0, "state_hash": "a"}),
            json!({"committed_madi": 7, "state_hash": "b"}),
        ];
        assert_eq!(last_committed_madi(&frames), json!(7));
    }

    const SAMPLE_MAEGIM_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.
"#;

    const SAMPLE_GUSEONG_SOURCE: &str = r#"
짜임 {
  형식: 진자_틀.
  입력 {
    기준점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

짜임 {
  형식: 물체_틀.
  입력 {
    붙는점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    꼭짓점: (수,수) <- (0.0, 0.0).
  }.
}.

a <- (L=1.0)인 진자_틀.
b <- (L=0.8)인 물체_틀.
a.기준점 <- b.꼭짓점.
"#;

    const SAMPLE_ALRIM_SOURCE: &str = r#"
"jump"라는 알림이 오면 {
  y <- 1.
}.

매틱:움직씨 = {
  "tick"라는 알림이 오면 {
    t <- t + 1.
  }.
}.

"reset"라는 알림이 오면 {
  y <- 0.
}.
"#;

    const SAMPLE_BLOCK_EDITOR_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  t:변수 <- 0.
  안내:글 = "초기".
}.

(시작)할때 {
  안내 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
  t < 3 일때 {
    t 보여주기.
  } 아니면 {
    g 보여주기.
  }.
}.

(x) x목록에 대해 {
  고르기:
    { x < 1 }인것: {
      "low" 보여주기.
    }
    아니면: {
      "high" 보여주기.
    }.
}.

고르기:
  t < 10 인 경우 {
    "small" 보여주기.
  }
  모든 경우 다룸.

되풀이 {
  "tick" 보여주기.
}.

너머 {
  "side" 보여주기.
}.

{ t < 2 }인 동안 {
  t <- t + 1.
}.

{ t == 2 }인것 바탕으로 아니면 {
  "bad" 보여주기.
} 맞으면 {
  "ok" 보여주기.
}.

{ t >= 0 }인것 다짐하고(알림) 아니면 {
  "warn" 보여주기.
}.

?? {
  "prompt" 보여주기.
}

"answer" ??: {
  "after" 보여주기.
}

{ t >= 0 }인것 ??
?? {
  "cond" 보여주기.
}

??:
  { t < 1 }인것: {
    "choose-low" 보여주기.
  }
??: {
    "choose-else" 보여주기.
  }

(기준:수 = 1) 판정:움직씨 = {
  기준 되돌림.
}.

기상청:임자 = {
  기상특보를 받으면 {
    알림.이름 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    정보.온도 보여주기.
  }.

  (알림 알림.이름 == "경보")인 알림을 받으면 {
    알림.정보.온도 보여주기.
  }.
}.

"tick"라는 알림이 오면 {
  "evt" 보여주기.
}

보개로 그려.

(매마디)마다 {
  t 톺아보기.
  되풀이 {
    t <- t + 1.
    { t > 0 }인것 일때 {
      멈추기.
    }.
  }.
}

실행정책 {
  실행모드: 일반.
  효과정책: 허용.
}.

짜임 {
  형식: 점_틀.
  입력 {
    시작점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

테스트:움직씨 = {
  보개마당 {
    #자막("정본 테스트").
  }.
}.

() 증명.
"#;

    const WAVE1_LESSONS: &[(&str, &str)] = &[
        (
            "synthetic/projectile-canon-surface.ddn",
            crate::self_contained_test_sources::PROJECTILE_CANON_SURFACE,
        ),
        (
            "synthetic/self-contained-multifile-entry.ddn",
            crate::self_contained_test_sources::MULTIFILE_ENTRY,
        ),
    ];

    fn wave1_lessons() -> Vec<(&'static str, String)> {
        WAVE1_LESSONS
            .iter()
            .map(|(path, source)| (*path, (*source).to_string()))
            .collect()
    }

    fn cli_canon_output(source: &str) -> crate::canon::CanonOutput {
        crate::canon::canonicalize(source, false).expect("cli canon")
    }

    fn cli_like_exec_policy_emit(source: &str) -> String {
        if !crate::canon::has_exec_policy_surface(source) {
            "{}\n".to_string()
        } else {
            cli_canon_output(source).exec_policy_map_json
        }
    }

    #[test]
    fn wasm_canon_flat_json_matches_cli_canon_surface() {
        let flat_json = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        assert!(flat_json.contains("\"schema\": \"ddn.guseong_flatten_plan.v1\""));
        assert!(flat_json.contains("\"name\": \"a\""));
        assert!(flat_json.contains("\"name\": \"b\""));
        assert!(flat_json.contains("\"dst_port\": \"기준점\""));
    }

    #[test]
    fn wasm_canon_flat_json_exact_parity_with_cli_output() {
        let wasm = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        let cli = cli_canon_output(SAMPLE_GUSEONG_SOURCE).guseong_flat_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_model_frontdoor_matches_shared_rust_canon_and_semantic_hash() {
        let source = r#"
낙하운동 = 모형 {
  시간:수@s :: (축, 처음::고정=0@s).
  위치:수@m :: (이어감, 처음::고정=100@m, 처음받음, 내줌).
  속도:수@m/s :: (들임, 내줌).
  위치' =:= 속도.
}.
"#;
        let wasm_canon = wasm_canon_ddn(source).expect("wasm model canon");
        let shared_canon =
            runtime_surface::canonicalize_shared_ddn(source).expect("shared model canon");
        assert_eq!(wasm_canon, shared_canon);

        let wasm_ir: JsonValue =
            serde_json::from_str(&wasm_model_ir_candidate_json(source).expect("wasm model IR"))
                .expect("wasm model IR json");
        let shared_ir: JsonValue = serde_json::from_str(
            &runtime_surface::model_ir_candidate_json_from_source(source).expect("shared model IR"),
        )
        .expect("shared model IR json");
        assert_eq!(wasm_ir["semantic_hash"], shared_ir["semantic_hash"]);
        assert_eq!(wasm_ir, shared_ir);
    }

    #[test]
    fn wasm_canon_boim_surface_is_not_legacy() {
        let source = r#"
(매마디)마다 {
  n <- 1.
  (n: n) 내보임.
}.
"#;
        assert!(!crate::canon::has_legacy_boim_surface(source));
        assert!(!crate::canon::has_legacy_boim_surface(
            SAMPLE_GUSEONG_SOURCE
        ));
    }

    #[test]
    fn wasm_run_summary_emits_ordered_successor_public_observation_trace() {
        let source = r#"
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
        let text = wasm_run_summary_json(source, "pack/ddn_boim_open_value_pack_w1/main.ddn", 2)
            .expect("wasm run summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["schema"], "ddn.public_observation_trace.v1");
        let names = trace["columns"]
            .as_array()
            .expect("columns")
            .iter()
            .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        assert_eq!(names, ["시간", "각도", "각속도"]);
        let samples = trace["samples"].as_array().expect("samples");
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0]["cells"][0]["value_text"], "1@s");
        assert_eq!(samples[0]["cells"][1]["value_text"], "17@rad");
        assert_eq!(samples[1]["cells"][2]["value_text"], "2");
        assert!(trace["columns"][0]["identity"]["source_ref"]
            .as_str()
            .unwrap_or_default()
            .starts_with("pack/ddn_boim_open_value_pack_w1/main.ddn#bytes="));
        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["presentation_state"], "ready");
        assert_eq!(
            projection["tables"][0]["columns"]
                .as_array()
                .expect("projection columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            names
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][0]["value_text"],
            "1@s"
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][1]["value_text"],
            "17@rad"
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["cells"][2]["value_text"],
            "2"
        );
        assert_eq!(
            projection["source_trace_hash"],
            summary["public_observation_trace_hash"]
        );
        assert_eq!(
            &summary["engine_state"]["public_observation_table_projection"],
            projection
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["state_hash"],
            summary["state_hash"]
        );
        assert!(summary.get("observation_manifest").is_none());
        assert!(summary["engine_state"]
            .get("observation_manifest")
            .is_none());
    }

    #[test]
    fn wasm_run_summary_keeps_multiple_and_repeated_authored_observations_as_separate_tables() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 10.
  횟수 <- 0.
}.

(매마디)마다 {
  되풀이 {
    (값: 시간, 증가: 1) 내보임.
    횟수 <- 횟수 + 1.
    { 횟수 >= 2 }인것 일때 {
      멈추기.
    }.
  }.
  (값: 각도, 다른: 시간) 내보임.
}.
"#;
        let text =
            wasm_run_summary_json(source, "app/main.ddn", 1).expect("multiple observation summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["samples"].as_array().map(Vec::len), Some(3));
        assert!(trace["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .all(|sample| sample["committed_madi"] == 0));

        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            projection["tables"][0]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][0]["columns"][1]["identity"]["name"],
            "증가"
        );
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][1]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][1]["columns"][1]["identity"]["name"],
            "다른"
        );
        assert_eq!(
            projection["tables"][1]["rows"].as_array().map(Vec::len),
            Some(1)
        );
        assert_ne!(
            projection["tables"][0]["columns"][0]["identity"]["source_ref"],
            projection["tables"][1]["columns"][0]["identity"]["source_ref"]
        );
        assert_eq!(projection["tables"][0]["rows"][0]["display_event_index"], 0);
        assert_eq!(projection["tables"][0]["rows"][1]["display_event_index"], 1);
        assert_eq!(projection["tables"][1]["rows"][0]["display_event_index"], 2);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm_vm_constructor_build_info_contains_pkg() {
        let vm = DdnWasmVm::new(SAMPLE_GUSEONG_SOURCE).expect("vm constructor");
        assert!(vm.get_build_info().contains("pkg="));
    }

    #[test]
    fn wasm_canon_maegim_plan_matches_cli_canon_surface() {
        let maegim_json = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        assert!(maegim_json.contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(maegim_json.contains("\"name\": \"g\""));
        assert!(maegim_json.contains("\"name\": \"theta0\""));
        assert!(maegim_json.contains("\"split_count_expr_canon\": \"24\""));
    }

    #[test]
    fn wasm_canon_maegim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        let cli = cli_canon_output(SAMPLE_MAEGIM_SOURCE).maegim_control_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_alrim_plan_matches_cli_canon_surface() {
        let alrim_json = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        assert!(alrim_json.contains("\"schema\": \"ddn.alrim_event_plan.v1\""));
        assert!(alrim_json.contains("\"kind\": \"jump\""));
        assert!(alrim_json.contains("\"kind\": \"tick\""));
        assert!(alrim_json.contains("\"scope\": \"root/seed:매틱\""));
    }

    #[test]
    fn wasm_canon_alrim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        let cli = cli_canon_output(SAMPLE_ALRIM_SOURCE).alrim_plan_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_exec_policy_map_uses_empty_fast_path_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert_eq!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_exec_policy_map_emits_schema_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert!(exec_json.contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert_ne!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_wave1_flat_json_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_flat_json(&source).expect("wasm flat json");
            let cli = cli_canon_output(&source).guseong_flat_json;
            assert_eq!(wasm, cli, "wave1 flat parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_maegim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_maegim_plan(&source).expect("wasm maegim plan");
            let cli = cli_canon_output(&source).maegim_control_json;
            assert_eq!(wasm, cli, "wave1 maegim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_alrim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_alrim_plan(&source).expect("wasm alrim plan");
            let cli = cli_canon_output(&source).alrim_plan_json;
            assert_eq!(wasm, cli, "wave1 alrim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_exec_policy_map_exact_parity_with_cli_emit_semantics() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_exec_policy_map(&source).expect("wasm exec policy map");
            let cli_like = cli_like_exec_policy_emit(&source);
            assert_eq!(wasm, cli_like, "wave1 exec parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_block_editor_plan_matches_cli_canon_surface() {
        // Report the actual source diagnostic before constructing a JS error
        // on a Native test host (wasm-bindgen cannot do that off wasm32).
        let _ = cli_canon_output(SAMPLE_BLOCK_EDITOR_SOURCE);
        let block_json =
            wasm_canon_block_editor_plan(SAMPLE_BLOCK_EDITOR_SOURCE).expect("wasm block plan");
        assert!(block_json.contains("\"schema\": \"ddn.block_editor_plan.v1\""));
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(block_json.contains("\"kind\": \"if_else\""));
        assert!(block_json.contains("\"kind\": \"for_each\""));
        assert!(block_json.contains("\"kind\": \"choose_else\""));
        assert!(block_json.contains("\"kind\": \"choose_exhaustive\""));
        assert!(block_json.contains("\"exhaustive\": \"true\""));
        assert!(block_json.contains("\"kind\": \"repeat\""));
        assert!(block_json.contains("\"kind\": \"open_block\""));
        assert!(block_json.contains("\"kind\": \"while_block\""));
        assert!(block_json.contains("\"kind\": \"contract_guard\""));
        assert!(block_json.contains("\"kind\": \"prompt_block\""));
        assert!(block_json.contains("\"kind\": \"prompt_after\""));
        assert!(block_json.contains("\"kind\": \"prompt_condition\""));
        assert!(block_json.contains("\"kind\": \"prompt_choose\""));
        assert!(block_json.contains("\"kind\": \"seed_def\""));
        assert!(block_json.contains("\"kind\": \"receive_block\""));
        assert!(block_json.contains("\"kind\": \"event_react\""));
        // Typed rendezvous is not a legacy flat block-editor node. Its
        // admission guard is checked separately below, without a JS exception
        // on this Native host.
        let transfer = crate::canon::canonicalize(
            "(기상청)의 (온도:1) 경보 ~~> 제.", false).err().unwrap();
        assert_eq!(transfer.code(), "E_CANON_TYPED_TRANSFER_REQUIRES_SHARED_FRONTDOOR");
        assert!(block_json.contains("\"kind\": \"return_value\""));
        assert!(block_json.contains("\"kind\": \"inspect_value\""));
        assert!(block_json.contains("\"kind\": \"bogae_draw\""));
        assert!(block_json.contains("\"kind\": \"break_loop\""));
        assert!(block_json.contains("\"kind\": \"bogae_madang_block\""));
        assert!(block_json.contains("\"kind\": \"exec_policy_block\""));
        assert!(block_json.contains("\"kind\": \"jjaim_block\""));
        assert!(block_json.contains("\"kind\": \"expr_stmt\""));
        assert!(block_json.contains("\"exprs\": {"));
        assert!(block_json.contains("\"kind\": \"call\""));
        // Preserve binding-node coverage separately from typed transfer,
        // which the flat editor intentionally does not admit.
        let binding_plan = wasm_canon_block_editor_plan("(값:1) 보여주기.")
            .expect("named argument block plan");
        assert!(binding_plan.contains("\"kind\": \"binding\""));
    }

    #[test]
    fn wasm_canon_block_editor_plan_preserves_authoring_hooks() {
        let source = r#"
채비 {
  t:변수 <- 0.
}.

(시작)할때 {
  t 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
}.
"#;
        let block_json = wasm_canon_block_editor_plan(source).expect("wasm block plan");
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(!block_json.contains("\"kind\": \"seed_def\""));
    }

    #[test]
    fn parse_stream_sidecar_tag_supports_ascii_and_ko_suffix() {
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_head"),
            Some(("energy_stream".to_string(), "head"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_capacity"),
            Some(("energy_stream".to_string(), "capacity"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("가격흐름_길이"),
            Some(("가격흐름".to_string(), "len"))
        );
        assert_eq!(parse_stream_sidecar_tag("plain_value"), None);
    }

    #[test]
    fn extract_ring_buffer_list_merges_sidecar_meta() {
        let value = ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(10)),
            ResourceValue::Fixed64(Fixed64::from_i64(20)),
        ]);
        let meta = StreamMeta {
            capacity: Some(4),
            head: Some(1),
            len: Some(2),
        };
        let stream = extract_ring_buffer(&value, Some(&meta)).expect("stream json");
        let obj = stream.as_object().expect("object");
        assert_eq!(obj.get("capacity").and_then(JsonValue::as_u64), Some(4));
        assert_eq!(obj.get("head").and_then(JsonValue::as_u64), Some(1));
        assert_eq!(obj.get("len").and_then(JsonValue::as_u64), Some(2));
        let buffer = obj
            .get("buffer")
            .and_then(JsonValue::as_array)
            .expect("buffer array");
        assert_eq!(buffer.len(), 4);
        assert!(buffer[2].is_null());
        assert!(buffer[3].is_null());
    }

    #[test]
    fn state_hash_is_stable_when_only_view_resource_changes() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("경제.자본".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(640));

        let view_prefixes = DEFAULT_VIEW_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect::<Vec<_>>();
        let state_hash_a = current_state_hash(&world, &view_prefixes);
        let view_hash_a = hash_json_string(&build_view_meta(world.world()));

        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(800));
        let state_hash_b = current_state_hash(&world, &view_prefixes);
        let view_hash_b = hash_json_string(&build_view_meta(world.world()));

        assert_eq!(state_hash_a, state_hash_b);
        assert_ne!(view_hash_a, view_hash_b);
    }

    fn point_resource(x: i64, y: i64) -> ResourceValue {
        ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(x)),
            ResourceValue::Fixed64(Fixed64::from_i64(y)),
        ])
    }

    fn draw_item_resource(kind: &str) -> ResourceValue {
        ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String(kind.to_string()),
                },
            ),
            (
                "x".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("x".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "y".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("y".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "r".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("r".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                },
            ),
        ]))
    }

    #[test]
    fn actual_logical_tetris_projects_canonical_rectangles() {
        // Match the existing Windows product stack budget in .cargo/config.toml;
        // do not raise the budget of the separate deep-owner regression tests.
        std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(|| {
        use crate::logical_input_tape::{LogicalInputBinding, LogicalInputSession, LogicalSnapshot};
        // The retired game-specific lesson is not an Alpha2+ product entry.
        // Execute the actual shipped D7 source through its logical frontdoor;
        // do not rewrite that source in a harness or invoke retired stdlib.
        let source = include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn");
        let binding: LogicalInputBinding = serde_json::from_str(include_str!(
            "../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
        let binding = LogicalInputSession::bind_source_for_fresh_run(source, binding).unwrap();
        let input = LogicalSnapshot { madi: 0, events: vec![], held: binding.initial_held.clone() };
        let mut session = LogicalInputSession::new(source, binding).unwrap();
        session.step(&input).expect("actual logical tetris execution");
        let before = session.committed_world().state_hash();
        let meta = build_view_meta(session.committed_world());
        let rows = meta["space2d"]["drawlist"].as_array().unwrap();
        // Empty board: one background rectangle plus the first four-cell piece.
        assert_eq!(rows.len(), 1 + 4);
        for row in rows {
            assert_eq!(row["kind"], "rect");
            let x = row["x"].as_f64().unwrap();
            let y = row["y"].as_f64().unwrap();
            assert_eq!(row["x1"], x);
            assert_eq!(row["y1"], y);
            assert_eq!(row["x2"], x + row["w"].as_f64().unwrap());
            assert_eq!(row["y2"], y + row["h"].as_f64().unwrap());
        }
        assert_eq!(session.committed_world().state_hash(), before);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn grid_drawlist_keeps_grid_and_space_projections_without_mutating_world() {
        let mut world = DetNuri::new();
        let cells = (0..64).map(|index| {
            let mut cell = draw_item_resource("rect");
            let ResourceValue::Map(fields) = &mut cell else {
                unreachable!()
            };
            fields.insert("id".to_string(), ResourceMapEntry {
                key: ResourceValue::String("id".to_string()),
                value: ResourceValue::String(format!("격자게임셀_{index}")),
            });
            cell
        }).collect();
        world.world_mut().set_resource_value(BOGAE_DRAWLIST_TAG.to_string(), ResourceValue::List(cells));
        world.world_mut().set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(32));
        world.world_mut().set_resource_fixed64(BOGAE_HEIGHT_TAG.to_string(), Fixed64::from_i64(32));
        let before = world.world().state_hash();
        let meta = build_view_meta(world.world());
        assert_eq!(meta["grid2d"]["schema"], "seamgrim.grid2d.v0");
        assert_eq!(meta["space2d"]["schema"], "seamgrim.space2d.v0");
        assert_eq!(meta["grid2d"]["drawlist"], meta["space2d"]["drawlist"]);
        assert_eq!(meta["space2d"]["drawlist"].as_array().unwrap().len(), 64);
        assert_eq!(meta["space2d"]["camera"]["x_max"], 32.0);
        assert_eq!(meta["primary"]["family"], "grid2d");
        assert_eq!(world.world().state_hash(), before);
    }

    #[test]
    fn build_view_meta_emits_primary_and_secondary_stack() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("space2d")
        );
        let secondary = obj
            .get("secondary")
            .and_then(JsonValue::as_array)
            .expect("secondary array");
        assert_eq!(secondary.len(), 1);
        assert_eq!(
            secondary[0]
                .as_object()
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("overlays")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_graph_only_sets_graph_primary() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("secondary")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_emits_exact_workbench_authoritative_results() {
        let mut world = DetNuri::new();
        for (tag, value) in [
            ("t", 3),
            ("x", 4),
            ("y", 5),
            ("vx", 6),
            ("vy", 7),
            ("a", 2),
            ("b", 1),
            ("theta", 1),
            ("omega", -1),
            ("energy", 9),
            ("kinetic_energy", 4),
            ("potential_energy", 5),
            ("L", 2),
            ("bob_x", 1),
            ("bob_y", -1),
        ] {
            world
                .world_mut()
                .set_resource_fixed64(tag.to_string(), Fixed64::from_i64(value));
        }
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "위치".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                    },
                ),
                (
                    "y".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("y".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                    },
                ),
            ])),
        );

        let meta = build_view_meta(world.world());
        let workbench = meta
            .get("workbench")
            .and_then(JsonValue::as_object)
            .expect("workbench result set");
        assert_eq!(
            workbench.get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_RESULT_SET_SCHEMA)
        );
        let results = workbench
            .get("results")
            .and_then(JsonValue::as_object)
            .expect("workbench results");
        assert_eq!(
            results.keys().cloned().collect::<Vec<_>>(),
            vec!["grid", "line", "pendulum", "projectile"]
        );
        assert_eq!(
            results["line"].get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_LINE_RESULT_SCHEMA)
        );
        assert_eq!(
            results["projectile"]
                .pointer("/content/current/vy")
                .and_then(JsonValue::as_f64),
            Some(7.0)
        );
        assert_eq!(
            results["pendulum"]
                .pointer("/content/current/physical_bob/y")
                .and_then(JsonValue::as_f64),
            Some(-1.0)
        );
        assert_eq!(
            results["grid"]
                .pointer("/content/position/x")
                .and_then(JsonValue::as_f64),
            Some(3.0)
        );
    }

    #[test]
    fn workbench_authoritative_results_are_exact_and_partial_inputs_fail_closed() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64("y".to_string(), Fixed64::from_i64(2));
        world
            .world_mut()
            .set_resource_fixed64("가로속도".to_string(), Fixed64::from_i64(3));
        world
            .world_mut()
            .set_resource_fixed64("세로속도".to_string(), Fixed64::from_i64(4));

        let meta = build_view_meta(world.world());
        assert!(meta.get("workbench").is_none());
    }

    #[test]
    fn serialize_world_resources_emits_value_json_alongside_value_keys() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "묶음값".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                    },
                ),
                (
                    "ok".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("ok".to_string()),
                        value: ResourceValue::Bool(true),
                    },
                ),
            ])),
        );

        let resources = serialize_world_resources(world.world());
        let root = resources.as_object().expect("resources object");
        let value = root
            .get("value")
            .and_then(JsonValue::as_object)
            .expect("value map");
        let value_json = root
            .get("value_json")
            .and_then(JsonValue::as_object)
            .expect("value_json map");

        assert!(value.get("묶음값").and_then(JsonValue::as_str).is_some());
        let row = value_json
            .get("묶음값")
            .and_then(JsonValue::as_object)
            .expect("value_json row");
        assert_eq!(row.get("x").and_then(JsonValue::as_f64), Some(1.0));
        assert_eq!(row.get("ok").and_then(JsonValue::as_bool), Some(true));
    }

    #[test]
    fn serialize_patch_set_resource_value_emits_value_json_payload() {
        let mut patch = Patch::default();
        patch.ops.push(PatchOp::SetResourceValue {
            tag: "리스트".to_string(),
            value: ResourceValue::List(vec![
                ResourceValue::Fixed64(Fixed64::from_i64(1)),
                ResourceValue::String("둘".to_string()),
            ]),
        });

        let json = serialize_patch(&patch);
        let rows = json.as_array().expect("patch array");
        assert_eq!(rows.len(), 1);
        let row = rows[0].as_object().expect("patch row");
        assert_eq!(
            row.get("op").and_then(JsonValue::as_str),
            Some("set_resource_value")
        );
        assert_eq!(row.get("tag").and_then(JsonValue::as_str), Some("리스트"));
        let value_json = row
            .get("value_json")
            .and_then(JsonValue::as_array)
            .expect("value_json array");
        assert_eq!(value_json.len(), 2);
        assert_eq!(value_json[0].as_f64(), Some(1.0));
        assert_eq!(value_json[1].as_str(), Some("둘"));
    }

    #[test]
    fn wo09_shared_patch_projection_preserves_both_payload_paths() {
        let mut patch = Patch::default();
        for length in 0..4 {
            patch.ops.push(PatchOp::SetResourceValue {
                tag: "출력".into(),
                value: ResourceValue::List((0..length)
                    .map(|n| ResourceValue::String(format!("행{n}")))
                    .collect()),
            });
        }
        let original = json!({
            "patch": serialize_patch(&patch),
            "state": {"patch": serialize_patch(&patch)},
        });
        let projected = serialize_patch(&patch);
        let shared = json!({
            "patch": projected.clone(),
            "state": {"patch": projected},
        });
        assert_eq!(original.to_string(), shared.to_string());
        assert_eq!(shared["patch"].as_array().unwrap().len(), 4);
        assert_eq!(shared["state"]["patch"], shared["patch"]);
    }

    #[test]
    fn restore_hash_diags_detect_mismatch_codes() {
        let payload = json!({
            "state_hash": "blake3:expected-state",
            "view_hash": "blake3:expected-view",
        });
        let diags = restore_hash_diags(&payload, "blake3:actual-state", "blake3:actual-view");
        assert_eq!(diags.len(), 2);
        let codes = diags
            .iter()
            .filter_map(|diag| diag.get("code").and_then(JsonValue::as_str))
            .collect::<Vec<_>>();
        assert!(codes.contains(&"STATE_HASH_MISMATCH"));
        assert!(codes.contains(&"VIEW_HASH_MISMATCH"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_layer_index_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polygon".to_string()),
                },
            ),
            (
                "레이어".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("레이어".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                },
            ),
            (
                "fill".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("fill".to_string()),
                    value: ResourceValue::String("#f59e0b".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(obj.get("kind").and_then(JsonValue::as_str), Some("polygon"));
        assert_eq!(
            obj.get("layer_index").and_then(JsonValue::as_f64),
            Some(3.0)
        );
        assert_eq!(obj.get("fill").and_then(JsonValue::as_str), Some("#f59e0b"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_group_id_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polyline".to_string()),
                },
            ),
            (
                "그룹".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("그룹".to_string()),
                    value: ResourceValue::String("pendulum.path".to_string()),
                },
            ),
            (
                "stroke".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("stroke".to_string()),
                    value: ResourceValue::String("#22c55e".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(
            obj.get("kind").and_then(JsonValue::as_str),
            Some("polyline")
        );
        assert_eq!(
            obj.get("group_id").and_then(JsonValue::as_str),
            Some("pendulum.path")
        );
        assert_eq!(
            obj.get("stroke").and_then(JsonValue::as_str),
            Some("#22c55e")
        );
    }

    #[test]
    fn bogae_kind_mapping_requires_an_exact_existing_identity() {
        assert_eq!(map_bogae_kind("선"), "line");
        assert_eq!(map_bogae_kind("사각형"), "rect");
        assert_eq!(map_bogae_kind("다각형"), "polygon");
        assert_eq!(map_bogae_kind("화살표"), "arrow");
        assert_eq!(map_bogae_kind("#보개/2D.Rect"), "rect");

        for unsupported in [
            "prefix-line",
            "line-suffix",
            "LINE",
            "사각형처럼",
            "원형",
            "sprite",
            "#보개/2D.Rect.extra",
            "#보개/2d.rect",
            "",
        ] {
            assert_eq!(map_bogae_kind(unsupported), unsupported);
        }
    }

    #[test]
    fn unit_value_wire_v2_is_explicit_and_roundtrips_ninth_axis() {
        let value = ResourceValue::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_i64(2),
            ddonirang_core::Unit::Mole,
        ));
        let encoded = resource_value_to_detjson(&value);
        assert_eq!(encoded["schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(
            encoded["dimension"]["schema"],
            ddonirang_core::UnitDim::WIRE_SCHEMA_V2
        );
        assert_eq!(
            encoded["dimension"]["axis_order"],
            json!(ddonirang_core::UnitDim::AXIS_ORDER_V2)
        );
        assert_eq!(
            encoded["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            resource_value_from_detjson(&encoded).expect("v2 roundtrip"),
            value
        );
    }

    #[test]
    fn wasm_snapshot_number_admission_matches_checked_textual_fixed64_boundary() {
        let three_halves_ulp: JsonValue =
            serde_json::from_str("0.000000000349245965480804443359375")
                .expect("three halves ULP JSON number");
        let overflow: JsonValue = serde_json::from_str("2147483648")
            .expect("out-of-range JSON number");

        assert_eq!(
            json_to_fixed64(&three_halves_ulp),
            None
        );
        assert_eq!(json_to_fixed64(&overflow), None);
        assert_eq!(
            json_to_fixed64(&json!("0.000000000349245965480804443359375")),
            Some(Fixed64::from_raw_i64(2))
        );
    }

    #[test]
    fn unit_value_wire_v2_rejects_v1_shape_unknown_version_and_axis_mismatch() {
        let legacy = json!({
            "type": "unit",
            "raw_i64": "2000000",
            "dimension": [0, 0, 0, 0, 0, 0, 0, 0],
        });
        assert!(resource_value_from_detjson(&legacy)
            .expect_err("unversioned v1 must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut unknown_version = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        unknown_version["schema"] = json!("ddn.unit_value_wire.v3");
        assert!(resource_value_from_detjson(&unknown_version)
            .expect_err("unknown version must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut reordered = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        reordered["dimension"]["axis_order"][0] = json!("time");
        reordered["dimension"]["axis_order"][1] = json!("length");
        assert!(resource_value_from_detjson(&reordered)
            .expect_err("axis reorder must fail")
            .contains("E_UNIT_DIMENSION_AXIS_ORDER_MISMATCH"));

        let mut missing_axis = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        missing_axis["dimension"]["exponents"]
            .as_array_mut()
            .expect("exponents")
            .pop();
        assert!(resource_value_from_detjson(&missing_axis)
            .expect_err("missing ninth axis must fail")
            .contains("E_UNIT_DIMENSION_AXIS_COUNT_MISMATCH"));
    }

    #[test]
    fn resource_snapshot_v2_rejects_checkpoint_unit_wire_mismatch() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let encoded = serialize_world_resources_snapshot(world.world());
        assert_eq!(encoded["schema"], RESOURCE_SNAPSHOT_SCHEMA_V2);
        assert_eq!(encoded["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);

        let mut mismatch = encoded.clone();
        mismatch["unit_wire_schema"] = json!("ddn.unit_value_wire.v1");
        let mut restored = DetNuri::new();
        assert!(restore_world_resources(restored.world_mut(), &mismatch)
            .expect_err("checkpoint mismatch must fail")
            .contains("E_UNIT_WIRE_CHECKPOINT_MISMATCH"));
    }

    fn wasm_restore_payload(resources: JsonValue) -> JsonValue {
        json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "state": {
                "input": {
                    "keys_pressed": 1,
                    "last_key_name": "m",
                    "pointer_x_i32": 2,
                    "pointer_y_i32": 3,
                    "rng_base_seed": 4,
                    // 입력 기록 identity 변경: 옛 raw/text 혼용이 아닌 정확 정본 기록.
                    // 이 fixture의 목적(restore 원자성/자원 보존)은 유지한다.
                    "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
                    "dt": "1",
                },
                "snapshot_v2": build_wasm_state_snapshot_v2(
                    resources,
                    json!({"속도": {"raw_i64": "2"}}),
                    // This unit fixture models seven committed boundaries.
                    // It is not execution/replay evidence; an empty history
                    // cannot legitimately carry next_tick_id=7.
                    (0..7).map(|madi| json!({
                        "committed_madi": madi,
                        "state_hash": format!("unit-fixture-state-{madi}"),
                        "input_snapshot": null,
                        "public_observations": [],
                    })).collect(),
                    7,
                    crate::bogae_time::SceneHistory::default().snapshot(),
                ),
            },
        })
    }

    #[test]
    fn wasm_state_snapshot_v2_roundtrips_ninth_axis_unit_exactly() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let resources = serialize_world_resources_snapshot(world.world());
        let payload = wasm_restore_payload(resources.clone());
        let staged = stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
            .expect("v2 state snapshot");

        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world()),
            resources
        );
        assert_eq!(staged.tick_id, 7);
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["raw_i64"],
            json!(Fixed64::from_i64(2).raw_i64().to_string())
        );
    }

    #[test]
    fn wasm_restore_fixture_never_accepts_unbacked_madi_cursor() {
        let resources = serialize_world_resources_snapshot(DetNuri::new().world());
        let mut payload = wasm_restore_payload(resources);
        payload["state"]["snapshot_v2"]["public_observation_frames"] = json!([]);
        assert!(stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
            .err().unwrap().starts_with("E_WASM_STATE_TICK_INVALID:"));
    }

    #[test]
    fn wasm_state_snapshot_v2_rejects_old_missing_and_mixed_versions() {
        let resources = serialize_world_resources_snapshot(DetNuri::new().world());
        let mut old = wasm_restore_payload(resources.clone());
        old["state"]["snapshot_v1"] = old["state"]["snapshot_v2"].take();
        assert!(
            stage_wasm_state_restore(&old, "app/main.ddn", "source", "canon")
                .err()
                .expect("v1 must fail")
                .contains("E_WASM_STATE_SNAPSHOT_MIXED_VERSION")
        );

        let mut missing = wasm_restore_payload(resources.clone());
        missing["state"]
            .as_object_mut()
            .expect("state")
            .remove("snapshot_v2");
        assert!(
            stage_wasm_state_restore(&missing, "app/main.ddn", "source", "canon")
                .err()
                .expect("missing snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );

        let mut unknown = wasm_restore_payload(resources);
        unknown["state"]["snapshot_v2"]["schema"] = json!("ddn.wasm_state_snapshot.v3");
        assert!(
            stage_wasm_state_restore(&unknown, "app/main.ddn", "source", "canon")
                .err()
                .expect("unknown snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );
    }

    #[test]
    fn failed_staged_restore_leaves_existing_world_unchanged() {
        let mut current = DetNuri::new();
        current.world_mut().set_resource_value(
            "기존".to_string(),
            ResourceValue::Fixed64(Fixed64::from_i64(9)),
        );
        let before = serialize_world_resources_snapshot(current.world());
        let before_hash = current
            .world()
            .state_hash_excluding_resource_prefixes(&[])
            .to_hex()
            .to_string();

        let mut corrupt = serialize_world_resources_snapshot(DetNuri::new().world());
        corrupt["fixed64"]["먼저_유효"] = json!({"raw_i64": "1000000"});
        corrupt["fixed64"]["뒤에_손상"] = json!({"raw_i64": "not-an-i64"});
        let payload = wasm_restore_payload(corrupt);
        assert!(
            stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
                .err()
                .expect("corrupt later entry must fail")
                .contains("fixed64[뒤에_손상]")
        );

        assert_eq!(serialize_world_resources_snapshot(current.world()), before);
        assert_eq!(
            current
                .world()
                .state_hash_excluding_resource_prefixes(&[])
                .to_hex()
                .to_string(),
            before_hash
        );
    }

    #[test]
    fn snapshot_param_overrides_use_the_shared_fixed64_raw_carrier() {
        let exact = Fixed64::from_raw_i64(2);
        let params = BTreeMap::from([
            ("고정".to_string(), Value::Fixed64(exact)),
            ("글".to_string(), Value::String("2".to_string())),
        ]);

        let serialized = serialize_param_overrides(&params);
        assert_eq!(serialized["고정"], json!({"raw_i64": "2"}));
        assert_eq!(serialized["글"], json!("2"));
        assert_eq!(parse_param_overrides(&serialized).expect("raw carrier restore"), params);

        let binary_number = parse_param_overrides(&json!({"고정": 2}));
        assert!(
            binary_number
                .err()
                .expect("binary JSON number must fail closed")
                .contains("E_WASM_STATE_PARAMS_INVALID")
        );
    }
