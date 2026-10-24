//! Existing tests; namespace and included source bytes preserved.
    #[test]
    fn wo24_model_head_missing_raw_record_returns_existing_error() {
        let raw = include_str!("../../../work/wo24_model_head_missing_raw.json");
        let source: VirtualProjectSource = serde_json::from_str(raw).unwrap();
        let before = serde_json::to_vec(&source).unwrap();
        let error = build_project_symbol_boundary(source.clone()).unwrap_err();
        assert_eq!(error.code, "E_PROJECT_MODULE_SOURCE_MISSING");
        assert_eq!(error.path.as_deref(), Some("lib/머리.ddn"));
        assert_eq!(serde_json::to_vec(&source).unwrap(), before);
    }

    #[test]
    fn wo24_model_head_control_preserves_r95_result_bytes() {
        let raw = include_str!("../../../work/wo24_model_head_control_raw.json");
        let expected = include_str!("../../../work/wo24_model_head_control_expected.json");
        let actual = build_project_symbol_boundary_json(raw).unwrap();
        assert_eq!(actual.as_bytes(), expected.as_bytes());
    }

    #[test]
    #[cfg(not(feature = "beta-project-features"))]
    fn alpha_scope_rejects_gaji_lifecycle_without_source_or_lock_changes() {
        let mut source = sample_multifile_source();
        source.files.push(VirtualProjectFileSource {
            path: "ddn.lock".into(), role: "lock".into(), utf8_text: "보존-lock-bytes".into(),
        });
        let artifact = build_local_gaji_artifact(extraction_request(sample_multifile_source())).unwrap();
        let before = serde_json::to_vec(&source).unwrap();
        for operation in [
            GajiLifecycleOperation::Install { alias: "힘".into(), artifact: artifact.clone() },
            GajiLifecycleOperation::Update { alias: "힘".into(), artifact: artifact.clone() },
            GajiLifecycleOperation::Remove { alias: "힘".into() },
        ] {
            let error = build_gaji_lifecycle_plan(lifecycle_request(source.clone(), operation)).unwrap_err();
            assert_eq!(error.code, "E_ALPHA_GAJI_LIFECYCLE_UNSUPPORTED");
            assert_eq!(serde_json::to_vec(&source).unwrap(), before);
        }
    }

    #[test]
    #[cfg(not(feature = "beta-project-features"))]
    fn alpha_scope_rejects_model_project_without_source_changes() {
        let source = sample_bounded_model_source();
        let before = serde_json::to_vec(&source).unwrap();
        let error = build_project_graph(source.clone()).unwrap_err();
        assert_eq!(error.code, "E_ALPHA_MODEL_PROJECT_UNSUPPORTED");
        assert_eq!(serde_json::to_vec(&source).unwrap(), before);
    }

    #[test]
    #[cfg(not(feature = "beta-project-features"))]
    fn alpha_scope_rejects_dataset_and_recipe_without_source_changes() {
        let source = sample_sealed_dataset_project();
        let before = serde_json::to_vec(&source).unwrap();
        assert_eq!(normalize_virtual_project_source(source.clone()).unwrap_err().code, "E_ALPHA_DATASET_UNSUPPORTED");
        assert_eq!(project_asset_registry_from_source(&source).unwrap_err().code, "E_ALPHA_DATASET_UNSUPPORTED");
        assert_eq!(build_project_execution_recipe(ProjectExecutionRecipeRequest {
            schema: PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA.into(),
            prepare_link_sha256: "0".repeat(64), recipe_version: 1, setup: json!({}),
        }).unwrap_err().code, "E_ALPHA_DATASET_UNSUPPORTED");
        assert_eq!(serde_json::to_vec(&source).unwrap(), before);
    }
    use super::*;

    #[test]
    #[cfg(not(feature = "beta-project-features"))]
    fn alpha_scope_rejects_each_dataset_role_and_keeps_unknown_role_rejection() {
        for role in ["asset_manifest", "sealed_dataset_envelope", "dataset_payload", "transform_recipe", "domain_evidence_manifest"] {
            let mut source = sample_source();
            source.files.push(VirtualProjectFileSource { path:"datasets/input.json".into(), role:role.into(), utf8_text:"{}".into() });
            let before = serde_json::to_vec(&source).unwrap();
            assert_eq!(normalize_virtual_project_source(source.clone()).unwrap_err().code, "E_ALPHA_DATASET_UNSUPPORTED");
            assert_eq!(serde_json::to_vec(&source).unwrap(), before);
        }
        let mut source = sample_source();
        source.files.push(VirtualProjectFileSource { path:"other.json".into(), role:"dataset_compat_fallback".into(), utf8_text:"{}".into() });
        assert_eq!(normalize_virtual_project_source(source).unwrap_err().code, "E_PROJECT_FILE_ROLE");
    }

    fn d9_source(root: &str, entry: &str, files: &[(&str, &str)]) -> VirtualProjectSource {
        let mut inputs = vec![VirtualProjectFileSource { path: "chosen-root.ddn".into(), role: "root".into(), utf8_text: root.into() }];
        inputs.extend(files.iter().map(|(path, text)| VirtualProjectFileSource {
            path: (*path).into(), role: "ddn_source".into(), utf8_text: (*text).into(),
        }));
        VirtualProjectSource { schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.into(), kind: Some(LOCAL_PROJECT_TREE_KIND.into()),
            package_hash: None, root_path: "chosen-root.ddn".into(), entry_path: entry.into(), files: inputs }
    }

    #[test]
    fn d9_canonical_named_start_runs_without_a_hidden_wrapper() {
        let source = d9_source("(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.",
            "app/main.ddn", &[("app/main.ddn", "실행:움직씨 := { 0 보여주기. }")]);
        let prepared = build_project_prepare_link(source.clone(), "", false, false).unwrap();
        assert_eq!(prepared.target, "실행");
        assert!(!prepared.linked_execution_source.contains("매마디:움직씨"));
        let run = build_project_run_summary(source, "", false, false).unwrap();
        assert_eq!(run.runtime_output_log_texts, vec!["0"]);
    }

    #[test]
    fn d9_multisource_exports_bind_after_all_definitions_are_collected() {
        let source = d9_source("(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=힘,머리=\"head.ddn\",원문=차림[\"head.ddn\",\"value.ddn\"])], 시작=(모듈=앱,정의=실행)) 얽음.",
            "main.ddn", &[("main.ddn", "(힘=힘) 쓰임. 실행:움직씨 := { 힘.중력 보여주기. }"),
                ("head.ddn", "(중력=중력) 드러냄."), ("value.ddn", "중력:수 := 9.8.")]);
        let run = build_project_run_summary(source, "", false, false).unwrap();
        assert_eq!(run.runtime_output_log_texts.len(), 1);
        assert!(run.runtime_output_log_texts[0].starts_with("9.8"));
    }

    #[test]
    fn d9_head_membership_and_startless_library_are_explicit() {
        let missing = d9_source("(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"head.ddn\",원문=\"value.ddn\")]) 얽음.", "",
            &[("head.ddn", ""), ("value.ddn", "값:수 := 1.")]);
        assert_eq!(build_project_graph(missing).unwrap_err().code, "E_PROJECT_HEAD_OUTSIDE_SOURCE_CLOSURE");
        let library = d9_source("(글판=\"v25\", 모듬=차림[(이름=도구,머리=\"lib.ddn\",원문=\"lib.ddn\")]) 얽음.", "", &[("lib.ddn", "값:수 := 1.")]);
        build_project_symbol_boundary(library.clone()).unwrap();
        assert_eq!(build_project_run_summary(library, "", false, false).unwrap_err().code, "E_PROJECT_START_REQUIRED");
    }

    #[test]
    fn d9_nested_root_directory_closure_and_rename_preserve_membership() {
        let mut source = d9_source("(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"src/main.ddn\",원문=\"src\")], 시작=(모듈=앱,정의=실행)) 얽음.",
            "nested/src/main.ddn", &[("nested/src/main.ddn", "실행:움직씨 := { 0 보여주기. }"),
                ("nested/src/value.ddn", "값 := 1.")]);
        source.root_path = "nested/custom.ddn".into();
        source.files[0].path = source.root_path.clone();
        let graph = build_project_graph(source.clone()).unwrap();
        assert_eq!(graph.module_sources["앱"].len(), 2);
        let hash = virtual_project_source_sha256(&source).unwrap();
        let plan = build_rename_file_plan(source, &hash, "nested/src/main.ddn", "nested/src/entry.ddn").unwrap();
        let graph = build_project_graph(plan.project_source).unwrap();
        assert_eq!(graph.entry_path, "nested/src/entry.ddn");
    }

    #[test]
    fn d9_import_cycles_are_rejected_before_execution() {
        let source = d9_source("(글판=\"v25\", 모듬=차림[(이름=도구,머리=\"a.ddn\",원문=\"a.ddn\"),(이름=자료,머리=\"b.ddn\",원문=\"b.ddn\")]) 얽음.", "",
            &[("a.ddn", "(자료=자료) 쓰임."), ("b.ddn", "(도구=도구) 쓰임.")]);
        let error = build_project_symbol_boundary(source).unwrap_err();
        assert_eq!(error.code, "E_IMPORT_CYCLE", "{error:?}");
    }

    #[test]
    fn wo03_project_epoch_is_owned_by_root_not_source_content() {
        for text in ["값 := 1. 값 보여주기.", "값 <- 1. 값 보여주기."] {
            let mut source = sample_source();
            source.files[0].utf8_text = text.to_string();
            assert_eq!(project_source_profile(&source, None).unwrap_err().code,
                "E_PROJECT_ROOT_PARSE");
            // This exercises epoch extraction only; module resolution is a separate gate.
            // D9: package version is not the source epoch; only 글판 owns it.
            source.files[1].utf8_text = "(가지=로컬.검사, 판본=\"1.0\", 글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")]) 얽음.".to_string();
            assert_eq!(project_source_profile(&source, None).unwrap(),
                crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY);
            assert_eq!(project_source_profile(&source, Some("legacy")).unwrap_err().code,
                "E_PROJECT_SOURCE_PROFILE_CONFLICT");
        }
    }

    #[test]
    fn wo03_project_epoch_rejects_unknown_duplicate_and_nonfield_values() {
        // D9 moves these failures into the common typed root parser. Preserve
        // each negative purpose by checking its specific underlying cause.
        for (root, cause) in [
            ("(글판=\"v24\") 얽음.", "E_WORK_ROOT_SOURCE_EPOCH"),
            ("(글판=\"v25\", 글판=\"v25\") 얽음.", "E_WORK_ROOT_DUPLICATE_FIELD"),
            ("(글판=v25) 얽음.", "E_WORK_ROOT_SOURCE_EPOCH"),
            ("() 얽음.", "E_WORK_ROOT_SOURCE_EPOCH"),
            ("프로젝트 { 판본: \"v25\". }.", "E_WORK_ROOT_SURFACE"),
            ("(가지=검사, 판본=\"v25\") 얽음.", "E_WORK_ROOT_SOURCE_EPOCH"),
            ("(글판=\"v25\") 얼개.", "E_WORK_ROOT_SURFACE"),
        ] {
            let mut source = sample_source();
            source.files[1].utf8_text = root.to_string();
            let error = project_source_profile(&source, None).unwrap_err();
            assert_eq!(error.code, "E_PROJECT_ROOT_PARSE", "{root}");
            assert!(error.message.contains(cause), "{root}: {error:?}");
        }
    }

    #[test]
    fn wo03_bom_source_is_rejected_without_changing_original_offsets() {
        let text = "\u{feff}값 := 1.\n값 보여주기.\n";
        let error = mask_module_metadata_blocks(text).expect_err("RD-534 rejects BOM");
        assert_eq!(error.code, "E_SOURCE_UTF8_BOM_FORBIDDEN");
        assert_eq!(error.source_span, Some((0, 3)));
        assert_eq!(error.phase, Some("source_ingress"));
        assert_eq!(text.find('값'), Some(3));
        assert_eq!(text.as_bytes().get(..3), Some(&[0xef, 0xbb, 0xbf][..]));
    }

    #[test]
    fn wo03_bom_project_root_and_module_are_rejected_at_ingress() {
        for role in ["root", "ddn_source"] {
            let mut source = sample_source();
            let file = source
                .files
                .iter_mut()
                .find(|file| file.role == role)
                .unwrap();
            file.utf8_text.insert(0, '\u{feff}');
            let path = file.path.clone();
            let error = normalize_virtual_project_source(source).expect_err("BOM rejected");
            assert_eq!(error.code, "E_SOURCE_UTF8_BOM_FORBIDDEN");
            assert_eq!(error.path.as_deref(), Some(path.as_str()));
            assert_eq!(error.source_span, Some((0, 3)));
        }
    }

    #[test]
    fn wo03_bom_free_metadata_mask_preserves_unicode_offsets_and_newlines() {
        for newline in ["\n", "\r\n"] {
            let text = format!("드러냄 {{ 값. }}{newline}값 <- 1.{newline}");
            let masked = mask_module_metadata_blocks(&text).expect("BOM-free source");
            assert_eq!(masked.len(), text.len());
            assert_eq!(masked.find("값 <-"), text.find("값 <-"));
            assert!(masked.ends_with(&format!("값 <- 1.{newline}")));
        }
        // U+FEFF inside a string is data, not a byte-ingress BOM.
        let text = "값 <- \"\u{feff}\".";
        assert_eq!(mask_module_metadata_blocks(text).unwrap(), text);
    }

    #[test]
    fn rd203_prelink_validation_keeps_entry_path_and_exact_field_segment() {
        let entry = "앱:움직씨 := {\n사람 := (이름: \"또니\").\n나이 := 사람.나이.\n(나이: 나이) 내보임.\n}.\n";
        let source = VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: PROJECT_ROOT_FILE.to_string(),
            entry_path: "main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: PROJECT_ROOT_FILE.to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: entry.to_string(),
                },
            ],
        };
        let err = build_project_run_summary(source, "앱", true, false)
            .expect_err("missing field must fail before linked execution");
        assert_eq!(err.code, "E_FIELD_MISSING");
        assert_eq!(err.path.as_deref(), Some("main.ddn"));
        let span = err.source_span.expect("field span");
        assert_eq!(&entry[span.0..span.1], "나이");
        let field_start = entry.find("사람.나이").unwrap() + "사람.".len();
        assert_eq!(span, (field_start, field_start + "나이".len()));
    }

    #[test]
    fn canonicalizes_disk_crlf_without_changing_lf_or_lone_cr() {
        assert_eq!(canonicalize_disk_text("a\r\nb\r\n".to_string()), "a\nb\n");
        assert_eq!(canonicalize_disk_text("a\nb\n".to_string()), "a\nb\n");
        assert_eq!(canonicalize_disk_text("a\rb\n".to_string()), "a\rb\n");
    }

    fn sample_source() -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "\"안녕\" 보여주기.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "프로젝트 { 이름: \"샘\". }.".to_string(),
                },
            ],
        }
    }

    #[test]
    fn empty_project_current_callable_runs_through_the_shared_frontdoor() {
        let mut source = sample_source();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "뿌리.ddn")
            .expect("empty-project root")
            .utf8_text = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".to_string();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("empty-project entry")
            .utf8_text = "실행:움직씨 := {\n  값:수 := 0.\n  값 보여주기.\n}.\n".to_string();

        let first =
            build_project_run_summary_with_supported_profile(source.clone(), "", false, false)
                .expect("current empty-project callable runs");
        let second = build_project_run_summary_with_supported_profile(source, "", false, false)
            .expect("repeated current empty-project callable runs");
        assert_eq!(first.runtime_output_log_texts, vec!["0"]);
        assert_eq!(first.runtime_state_hash, second.runtime_state_hash);
        assert_eq!(first.summary_sha256, second.summary_sha256);
    }

    #[test]
    fn current_workbench_physics_body_runs_through_the_shared_frontdoor() {
        // D9: use the authored Workbench template, not a hidden callable wrapper.
        let source = VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\"), (이름=힘, 머리=\"lib/힘.ddn\", 원문=\"lib/힘.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(힘=힘) 쓰임.\n\n실행:움직씨 := {\n초기속도x:수 := 0.\n초기속도y:수 := 20.\n시간간격:수 := 0.1.\n중력가속도:수 := 힘.중력.\n속도:수 := 초기속도y - 중력가속도 * 시간간격.\n위치:수 := 초기속도x + 속도 * 시간간격.\n위치 보여주기.\n속도 보여주기.\n힘.중력 보여주기.\n}.\n".to_string(),
                },
                VirtualProjectFileSource {
                    path: "lib/힘.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(중력=중력) 드러냄.\n\n중력:수 := 9.8.\n".to_string(),
                },
            ],
        };

        let prepared = build_project_prepare_link(source.clone(), "", true, false)
            .expect("current Workbench physics body prepares through the shared frontdoor");
        assert_eq!(prepared.target, "실행");
        assert!(!prepared.linked_execution_source.contains("매마디:움직씨"));

        let first = build_project_run_summary_with_supported_profile(
            source.clone(),
            "",
            false,
            false,
        )
        .expect("current Workbench physics body runs");
        let second = build_project_run_summary_with_supported_profile(source, "", false, false)
            .expect("repeated current Workbench physics body runs");
        assert_eq!(first.runtime_state_hash, second.runtime_state_hash);
        assert_eq!(first.summary_sha256, second.summary_sha256);
        assert!(!first.runtime_output_log_texts.is_empty());
    }

    #[test]
    fn current_project_run_rejects_legacy_l11_before_runtime_publication() {
        let source = VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\"),(이름=운동,머리=\"lib/운동.ddn\",원문=\"lib/운동.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "쓰임 { 운동: \"./lib/운동\". }\n".to_string(),
                },
                VirtualProjectFileSource {
                    path: "lib/운동.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(속도=속도) 드러냄.\n\n속도:수 := 7.\n".to_string(),
                },
            ],
        };

        let error = build_project_run_summary_with_supported_profile(source, "앱", false, false)
            .expect_err("legacy brace directives must fail before a run summary exists");
        assert_eq!(error.code, "E_V25_DEFINITION_MIGRATION_REQUIRED");
        assert_eq!(error.path.as_deref(), Some("app/main.ddn"));

        let comment_only = VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "앱:움직씨 := { 값:수 := 1. // 쓰임 { 드러냄 { 공개 { 는 주석입니다\n값 보여주기.\n}.".to_string(),
                },
            ],
        };
        let summary = build_project_run_summary_with_supported_profile(comment_only, "앱", false, false)
            .expect("comment text must not trigger a legacy L11 rejection");
        assert_eq!(summary.runtime_output_log_texts, vec!["1"]);
    }

    #[test]
    fn supported_project_run_binds_declared_host_resource_before_execution() {
        let text = "프로젝트 입력\n";
        let request = HostResourceRequest {
            schema: host_resource::HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
            mode: "record".to_string(),
            resources: vec![host_resource::HostResourcePayload {
                logical_resource_id: "caption".to_string(),
                sha256: sha256_hex(text.as_bytes()),
                byte_length: text.len() as u64,
                media_type: host_resource::V1_TEXT_MEDIA_TYPE.to_string(),
                utf8_text: text.to_string(),
            }],
            transcript: None,
        };
        let summary = build_project_run_summary_with_supported_profile_and_host_resources(
            current_typed_multifile_source(),
            "앱",
            false,
            false,
            &request,
        )
        .expect("project host resource run");
        assert!(summary.host_resource_receipt.is_some());
        assert_ne!(
            summary.summary_sha256,
            build_project_run_summary_with_supported_profile(
                current_typed_multifile_source(),
                "앱",
                false,
                false
            )
            .expect("ordinary project run")
            .summary_sha256
        );

        let invalid = HostResourceRequest {
            schema: host_resource::HOST_RESOURCE_REQUEST_SCHEMA.to_string(),
            mode: "record".to_string(),
            resources: vec![host_resource::HostResourcePayload {
                logical_resource_id: "caption".to_string(),
                sha256: sha256_hex(text.as_bytes()),
                byte_length: 0,
                media_type: host_resource::V1_TEXT_MEDIA_TYPE.to_string(),
                utf8_text: text.to_string(),
            }],
            transcript: None,
        };
        let error = build_project_run_summary_with_supported_profile_and_host_resources(
            current_typed_multifile_source(),
            "앱",
            false,
            false,
            &invalid,
        )
        .expect_err("invalid declaration must stop before project execution");
        assert_eq!(error.code, "E_HOST_RESOURCE_PREFLIGHT");
    }

    fn sample_multifile_source() -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    // D9: explicit module membership and authored executable definition.
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\"),(이름=운동,머리=\"lib/운동.ddn\",원문=\"lib/운동.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(운동=운동) 쓰임.\n앱:움직씨 := { 운동.속도 보여주기. }.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "lib/운동.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(속도=속도) 드러냄.\n속도:수 := 7.".to_string(),
                },
            ],
        }
    }

    fn current_typed_multifile_source() -> VirtualProjectSource {
        let mut source = sample_multifile_source();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("current entry")
            .utf8_text = "(운동=운동) 쓰임.\n\n앱:움직씨 := { 운동.속도 보여주기. }.".to_string();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "lib/운동.ddn")
            .expect("current module")
            .utf8_text = "(속도=속도) 드러냄.\n\n속도:수 := 7.".to_string();
        source
    }

    fn sample_natural_iteration_multifile_source() -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\"),(이름=계산,머리=\"lib/계산.ddn\",원문=\"lib/계산.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: r#"(계산=계산) 쓰임.
앱:움직씨 := {
  채비 {
    목록:(수)차림 := (1, 2, 3) 차림.
    합:수 := 0.
  }.
  목록마다 { () 계산.누적하기. }.
  (합: 합) 내보임.
}"#.to_string(),
                },
                VirtualProjectFileSource {
                    path: "lib/계산.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: r#"(누적=누적) 드러냄.
(값:수) 누적:움직씨 := {
  합 <- 합 + 값.
}"#.to_string(),
                },
            ],
        }
    }

    #[test]
    fn natural_iteration_current_item_runs_after_multifile_linking() {
        let summary = build_project_run_summary(
            sample_natural_iteration_multifile_source(),
            "앱",
            true,
            false,
        )
        .expect("linked natural iteration run");
        assert_eq!(summary.runtime_final_row["합"], json!(6.0));
        assert_eq!(
            summary.project_observation_trace["samples"][0]["cells"][0]["value_text"],
            json!("6")
        );
        assert_eq!(
            summary.project_observation_table_projection["tables"][0]["rows"][0]["cells"][0]
                ["value_text"],
            json!("6")
        );
        assert_eq!(
            summary.runtime_engine_state["state"]["snapshot_v2"]["schema"],
            json!("ddn.wasm_state_snapshot.v2")
        );
        assert!(summary.linked_execution_source.contains("목록마다"));
        assert!(summary
            .linked_execution_canonical_ddn
            .contains("(목록):값 누적기."), "{}", summary.linked_execution_canonical_ddn);
        assert!(!summary.linked_execution_canonical_ddn.contains("목록마다"));
    }

    #[test]
    fn current_item_diagnostic_preserves_primary_code_path_and_span_at_project_boundary() {
        let mut source = sample_natural_iteration_multifile_source();
        let helper = source
            .files
            .iter_mut()
            .find(|file| file.path == "lib/계산.ddn")
            .expect("helper module");
        helper.utf8_text = helper
            .utf8_text
            .replace("(값:수) 누적:움직씨", "(왼:수, 오른:수) 누적:움직씨");
        let entry_text = source
            .files
            .iter()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry module")
            .utf8_text
            .clone();

        let err = build_project_prepare_link(source, "앱", true, false)
            .expect_err("ambiguous current item must fail before execution");
        assert_eq!(err.code, "E_CURRENT_ITEM_AMBIGUOUS");
        assert_eq!(err.path.as_deref(), Some("app/main.ddn"));
        assert!(
            err.source_span.is_some(),
            "source span must cross project boundary"
        );
        let (start, end) = err.source_span.expect("source span");
        assert!(entry_text[start..end].contains("계산.누적하기"));
        let wire = err.to_json_value();
        assert_eq!(wire["code"], "E_CURRENT_ITEM_AMBIGUOUS");
        assert_eq!(wire["source_path"], "app/main.ddn");
        assert!(wire["source_span"]["start"].is_u64());
        assert!(wire["source_span"]["end"].is_u64());
        assert_eq!(wire["phase"], "execution_mode_decision");
        assert_eq!(wire["cause"]["code"], "E_CURRENT_ITEM_AMBIGUOUS");
    }

    #[test]
    fn outside_iteration_required_pin_matches_shared_primary_diagnostic() {
        let mut source = sample_natural_iteration_multifile_source();
        let entry = source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry module");
        entry.utf8_text = entry
            .utf8_text
            .replace("목록마다 { () 계산.누적하기. }.", "() 계산.누적하기.");
        let entry_text = entry.utf8_text.clone();

        let err = build_project_prepare_link(source, "앱", true, false)
            .expect_err("outside-iteration required pin must fail");
        assert_eq!(err.code, "E_CALL_REQUIRED_PIN_MISSING", "{err:?}");
        assert_eq!(err.path.as_deref(), Some("app/main.ddn"));
        assert!(
            err.source_span.is_some(),
            "source span must cross project boundary"
        );
        let (start, end) = err.source_span.expect("source span");
        assert!(entry_text[start..end].contains("계산.누적하기"));
    }

    #[test]
    fn run_summary_preserves_the_same_structured_current_item_diagnostic_as_prepare_link() {
        runtime_surface::reset_run_summary_diagnostic_invocations();
        let mut source = sample_natural_iteration_multifile_source();
        let entry = source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry module");
        entry.utf8_text = entry
            .utf8_text
            .replace("목록마다 { () 계산.누적하기. }.", "() 계산.누적하기.");
        let entry_text = entry.utf8_text.clone();

        let prepare = build_project_prepare_link(source.clone(), "앱", true, false)
            .expect_err("prepare/link must reject outside-iteration current item");
        let run = build_project_run_summary(source, "앱", true, false)
            .expect_err("run-summary must reject outside-iteration current item");

        assert_eq!(run.code, prepare.code);
        assert_eq!(run.message, prepare.message);
        assert_eq!(run.path, prepare.path);
        assert_eq!(run.source_span, prepare.source_span);
        assert_eq!(run.phase, prepare.phase);
        assert_eq!(run.cause, prepare.cause);
        assert_eq!(run.code, "E_CALL_REQUIRED_PIN_MISSING");
        let (start, end) = run.source_span.expect("run-summary source span");
        assert!(entry_text[start..end].contains("계산.누적하기"));

        let wire = run.to_json_value();
        assert_eq!(wire["code"], "E_CALL_REQUIRED_PIN_MISSING");
        assert_eq!(wire["source_path"], "app/main.ddn");
        assert_eq!(wire["phase"], "execution_mode_decision");
        assert_eq!(wire["cause"]["code"], "E_CALL_REQUIRED_PIN_MISSING");
        assert!(wire["source_span"]["start"].is_u64());
        assert!(wire["source_span"]["end"].is_u64());
        assert_eq!(
            runtime_surface::run_summary_diagnostic_invocations(),
            1,
            "a failed project run must not retry through another runtime"
        );
    }

    #[test]
    fn run_summary_preserves_ambiguous_current_item_diagnostic_and_is_deterministic() {
        let mut source = sample_natural_iteration_multifile_source();
        let helper = source
            .files
            .iter_mut()
            .find(|file| file.path == "lib/계산.ddn")
            .expect("helper module");
        helper.utf8_text = helper
            .utf8_text
            .replace("(값:수) 누적:움직씨", "(왼:수, 오른:수) 누적:움직씨");

        let prepare = build_project_prepare_link(source.clone(), "앱", true, false)
            .expect_err("prepare/link must reject ambiguous current item");
        let first = build_project_run_summary(source.clone(), "앱", true, false)
            .expect_err("run-summary must reject ambiguous current item");
        let second = build_project_run_summary(source, "앱", true, false)
            .expect_err("repeated run-summary must reject without fallback");

        assert_eq!(first.code, "E_CURRENT_ITEM_AMBIGUOUS");
        assert_eq!(first.code, prepare.code);
        assert_eq!(first.path, prepare.path);
        assert_eq!(first.source_span, prepare.source_span);
        assert_eq!(first.phase, prepare.phase);
        assert_eq!(first.cause, prepare.cause);
        assert_eq!(first.to_json_value(), second.to_json_value());
        assert_eq!(
            first.to_json_value()["cause"]["code"],
            "E_CURRENT_ITEM_AMBIGUOUS"
        );
    }

    fn sample_bounded_model_source() -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "프로젝트 { 이름: \"정확한 관계 모형\". 첫글: \"./app/main.ddn\". 목표 { 앱: \"./app/main.ddn\". } }.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: crate::self_contained_test_sources::EXACT_ACYCLIC_MODEL.to_string(),
                },
            ],
        }
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn prepares_bounded_model_project_without_legacy_preprocess() {
        let prepared = build_project_prepare_link(sample_bounded_model_source(), "앱", true, false)
            .expect("typed model project prepare/link");
        assert_eq!(
            prepared.runtime_execution_envelope["backend_descriptor"]["backend_id"],
            crate::model_runtime::MODEL_BACKEND_ID
        );
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["mode"],
            "oneshot"
        );
        assert!(prepared.linked_execution_source.contains("검산 = 모형"));
        assert!(prepared.symbol_boundary.imports.is_empty());
        assert!(prepared.symbol_boundary.exports.is_empty());
    }

    fn sample_sealed_dataset_project() -> VirtualProjectSource {
        use crate::sealed_domain_dataset::{
            canonical_sealed_domain_dataset_json, seal_domain_dataset, DatasetCell, DatasetColumn,
            DatasetDtype, DatasetPayload, DatasetRow, DatasetScalar, DatasetSource,
            SealedDomainDataset, SEALED_DOMAIN_DATASET_SCHEMA,
        };

        let payload = "month,expense\n1,700\n2,\n";
        let dataset = seal_domain_dataset(
            SealedDomainDataset {
                schema: SEALED_DOMAIN_DATASET_SCHEMA.to_string(),
                artifact_id: "fixture.project-runtime".to_string(),
                artifact_version: "v1".to_string(),
                source: DatasetSource::ExternalSnapshot {
                    provider: "W1 fixture provider".to_string(),
                    title: "project runtime fixture".to_string(),
                    canonical_uri: "https://example.invalid/ddn/project-runtime".to_string(),
                    published_or_version: "fixture-v1".to_string(),
                    retrieved_at: "2026-07-27".to_string(),
                    license_id: "LicenseRef-W1-fixture".to_string(),
                    attribution: "W1 project runtime fixture".to_string(),
                },
                payload: DatasetPayload {
                    object_ref: "datasets/project-runtime/payload.csv".to_string(),
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
                ordered_rows: vec![DatasetRow {
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
                }],
                content_sha256: String::new(),
                schema_sha256: String::new(),
            },
            payload.as_bytes(),
        )
        .expect("seal project dataset");
        let envelope = canonical_sealed_domain_dataset_json(&dataset).expect("canonical envelope");
        let bundle_id = "ddn.w1.project-runtime";
        let envelope_path = "datasets/project-runtime/envelope.json";
        let payload_path = "datasets/project-runtime/payload.csv";
        let envelope_handle =
            ddonirang_core::asset_handle_from_bundle_path(bundle_id, envelope_path).raw();
        let payload_handle =
            ddonirang_core::asset_handle_from_bundle_path(bundle_id, payload_path).raw();
        let manifest = serde_json::to_string(&json!({
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
                    "hash": blake3::hash(payload.as_bytes()).to_hex().to_string(),
                    "size": payload.len(),
                    "mime": "text/csv"
                }
            ]
        }))
        .expect("asset manifest");
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "프로젝트 { 이름: \"봉인 자료\". 첫글: \"./app/main.ddn\". 목표 { 앱: \"./app/main.ddn\". } }.".to_string(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: r#"
매틱:움직씨 = {
  자료값 <- (@"datasets/project-runtime/envelope.json") 자료.열기.
  첫행 <- (자료값.행, 0) 차림.값.
  첫셀 <- (첫행.cells, 0) 차림.값.
  (월: 첫셀.value.value) 내보임.
}
"#
                    .to_string(),
                },
                VirtualProjectFileSource {
                    path: "ddn.asset.json".to_string(),
                    role: "asset_manifest".to_string(),
                    utf8_text: manifest,
                },
                VirtualProjectFileSource {
                    path: envelope_path.to_string(),
                    role: "sealed_dataset_envelope".to_string(),
                    utf8_text: envelope,
                },
                VirtualProjectFileSource {
                    path: payload_path.to_string(),
                    role: "dataset_payload".to_string(),
                    utf8_text: payload.to_string(),
                },
            ],
        }
    }

    fn sample_boim_multifile_source() -> VirtualProjectSource {
        VirtualProjectSource {
            schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
            kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
            package_hash: None,
            root_path: "뿌리.ddn".to_string(),
            entry_path: "app/main.ddn".to_string(),
            files: vec![
                VirtualProjectFileSource {
                    path: "뿌리.ddn".to_string(),
                    role: "root".to_string(),
                    utf8_text: "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\"),(이름=도움,머리=\"lib/도움.ddn\",원문=\"lib/도움.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".into(),
                },
                VirtualProjectFileSource {
                    path: "app/main.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(도움=도움) 쓰임.\n시간 := 0.\n각도 := 도움.시작값.\n앱:움직씨 := {\n(매마디)마다 {\n시간 <- 시간 + 1.\n각도 <- 각도 + 도움.증가값.\n(경과: 시간 * 1@s, 각도: 각도 * 1@rad, 증가량: 도움.증가값) 내보임.\n}.\n}.".into(),
                },
                VirtualProjectFileSource {
                    path: "lib/도움.ddn".to_string(),
                    role: "ddn_source".to_string(),
                    utf8_text: "(시작값=시작값, 증가값=증가값) 드러냄.\n시작값 := 20.\n증가값 := 3.".into(),
                },
            ],
        }
    }

    #[test]
    fn normalizes_paths_and_hashes_deterministically() {
        let first = normalize_virtual_project_source(sample_source()).expect("normalize");
        let second = normalize_virtual_project_source(sample_source()).expect("normalize");
        assert_eq!(
            first.canonical_manifest_sha256,
            second.canonical_manifest_sha256
        );
        assert_eq!(first.files[0].path, "app/main.ddn");
    }

    #[test]
    fn accepts_current_portable_package_hash_without_using_it_as_execution_identity() {
        let baseline =
            normalize_virtual_project_source(sample_source()).expect("baseline project source");
        let mut portable = sample_source();
        portable.package_hash = Some(String::new());
        let reopened =
            normalize_virtual_project_source(portable).expect("current portable project source");
        assert_eq!(
            reopened.canonical_manifest_sha256,
            baseline.canonical_manifest_sha256
        );
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn accepts_only_the_sealed_dataset_portable_file_roles_added_by_w1() {
        let mut source = sample_source();
        source.files.extend([
            VirtualProjectFileSource {
                path: "ddn.asset.json".to_string(),
                role: "asset_manifest".to_string(),
                utf8_text: "{}".to_string(),
            },
            VirtualProjectFileSource {
                path: "datasets/economy/envelope.json".to_string(),
                role: "sealed_dataset_envelope".to_string(),
                utf8_text: "{}".to_string(),
            },
            VirtualProjectFileSource {
                path: "datasets/economy/payload.csv".to_string(),
                role: "dataset_payload".to_string(),
                utf8_text: "month,expense\n1,700\n".to_string(),
            },
            VirtualProjectFileSource {
                path: "datasets/economy/recipe.json".to_string(),
                role: "transform_recipe".to_string(),
                utf8_text: "{}".to_string(),
            },
        ]);
        let normalized =
            normalize_virtual_project_source(source.clone()).expect("sealed portable roles");
        assert!(normalized
            .files
            .iter()
            .any(|file| file.role == "asset_manifest"));
        assert!(normalized
            .files
            .iter()
            .any(|file| file.role == "sealed_dataset_envelope"));
        assert!(normalized
            .files
            .iter()
            .any(|file| file.role == "dataset_payload"));
        assert!(normalized
            .files
            .iter()
            .any(|file| file.role == "transform_recipe"));

        source.files.push(VirtualProjectFileSource {
            path: "datasets/economy/compat.json".to_string(),
            role: "dataset_compat_fallback".to_string(),
            utf8_text: "{}".to_string(),
        });
        let err = normalize_virtual_project_source(source).expect_err("unknown role must fail");
        assert_eq!(err.code, "E_PROJECT_FILE_ROLE");
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn project_normalize_validates_domain_evidence_role_against_dataset_rows_and_columns() {
        use crate::domain_evidence_manifest::{
            seal_domain_evidence_manifest, DomainEvidenceManifest,
        };

        let mut source = sample_sealed_dataset_project();
        let envelope = source
            .files
            .iter()
            .find(|file| file.role == "sealed_dataset_envelope")
            .expect("dataset envelope");
        let dataset: SealedDomainDataset =
            serde_json::from_str(&envelope.utf8_text).expect("dataset");
        let evidence_context = vec![EvidenceDatasetTarget {
            dataset_content_sha256: dataset.content_sha256.clone(),
            project_revision_ref: None,
            column_names: dataset
                .ordered_columns
                .iter()
                .map(|column| column.name.clone())
                .collect(),
            rows: dataset
                .ordered_rows
                .iter()
                .map(|row| EvidenceDatasetRowTarget {
                    source_ordinal: row.source_ordinal,
                    cell_statuses: row
                        .cells
                        .iter()
                        .map(|_| EvidenceCellStatus::Present)
                        .collect(),
                })
                .collect(),
        }];
        let zero_hash = "0".repeat(64);
        let manifest = json!({
            "schema": "ddn.domain_evidence_manifest.v1",
            "subject": {
                "dataset_content_sha256": dataset.content_sha256,
                "project_revision_ref": "revision:project-normalize"
            },
            "sources": [
                {
                    "source_ref": "source:budget",
                    "title": "예산 원자료",
                    "provider": "W1 fixture provider",
                    "canonical_identifier": "https://example.invalid/ddn/budget-source",
                    "published_or_version": "fixture-v1",
                    "retrieved_at": "2026-07-28",
                    "rights": {
                        "license_id": "LicenseRef-W1-fixture",
                        "holder": "W1 fixture holder",
                        "reuse_mode": "reference"
                    },
                    "payload_mode": "reference"
                }
            ],
            "claims": [
                {
                    "target": {
                        "dataset_content_sha256": dataset.content_sha256,
                        "row_source_ordinal": 0,
                        "column_name": "지출"
                    },
                    "relation": "direct_source",
                    "source_ref": "source:budget",
                    "source_locator": "csv:row:1:column:2"
                }
            ],
            "activities": [],
            "hashes": {
                "evidence_manifest_sha256": zero_hash,
                "content_binding_sha256": zero_hash
            }
        });
        let typed_manifest: DomainEvidenceManifest =
            serde_json::from_value(manifest).expect("typed evidence manifest");
        let sealed_manifest = seal_domain_evidence_manifest(typed_manifest, &evidence_context)
            .expect("seal evidence");
        source.files.push(VirtualProjectFileSource {
            path: "evidence/budget.manifest.json".to_string(),
            role: "domain_evidence_manifest".to_string(),
            utf8_text: serde_json::to_string(&sealed_manifest).expect("manifest"),
        });

        let normalized =
            normalize_virtual_project_source(source.clone()).expect("valid evidence project");
        assert!(normalized
            .files
            .iter()
            .any(|file| file.role == "domain_evidence_manifest"));
        let prepared =
            build_project_prepare_link(source.clone(), "앱", true, true).expect("prepare evidence");
        assert_eq!(prepared.domain_evidence_manifests.len(), 1);
        assert_eq!(
            prepared.runtime_execution_envelope["domain_evidence_manifests"][0]
                ["content_binding_sha256"],
            prepared.domain_evidence_manifests[0].content_binding_sha256
        );
        let run =
            build_project_run_summary(source.clone(), "앱", true, true).expect("run evidence");
        assert_eq!(
            run.domain_evidence_manifests,
            prepared.domain_evidence_manifests
        );
        assert_eq!(
            run.execution_preparation["domain_evidence_manifests"][0]["evidence_manifest_sha256"],
            prepared.domain_evidence_manifests[0].evidence_manifest_sha256
        );

        let mut evidence_changed = source.clone();
        let changed_file = evidence_changed
            .files
            .iter_mut()
            .find(|file| file.role == "domain_evidence_manifest")
            .expect("changed evidence file");
        let mut changed_manifest: DomainEvidenceManifest =
            serde_json::from_str(&changed_file.utf8_text).expect("changed manifest");
        changed_manifest.claims[0].note = Some("evidence-only revision".to_string());
        changed_manifest =
            seal_domain_evidence_manifest(changed_manifest, &evidence_context).expect("reseal");
        changed_file.utf8_text =
            serde_json::to_string(&changed_manifest).expect("changed manifest JSON");
        let changed_prepared =
            build_project_prepare_link(evidence_changed.clone(), "앱", true, true)
                .expect("changed prepare");
        let changed_run =
            build_project_run_summary(evidence_changed, "앱", true, true).expect("changed run");
        assert_ne!(
            changed_prepared.domain_evidence_manifests[0].evidence_manifest_sha256,
            prepared.domain_evidence_manifests[0].evidence_manifest_sha256
        );
        assert_ne!(
            changed_prepared.domain_evidence_manifests[0].content_binding_sha256,
            prepared.domain_evidence_manifests[0].content_binding_sha256
        );
        assert_ne!(
            changed_prepared.project_graph.project_graph_sha256,
            prepared.project_graph.project_graph_sha256
        );
        assert_ne!(
            changed_prepared.prepare_link_sha256,
            prepared.prepare_link_sha256
        );
        assert_ne!(changed_run.summary_sha256, run.summary_sha256);
        assert_eq!(changed_run.runtime_state_hash, run.runtime_state_hash);
        assert_eq!(changed_run.bounded_state_sha256, run.bounded_state_sha256);

        let evidence_file = source
            .files
            .iter_mut()
            .find(|file| file.role == "domain_evidence_manifest")
            .expect("evidence file");
        let mut invalid: JsonValue =
            serde_json::from_str(&evidence_file.utf8_text).expect("manifest value");
        invalid["claims"][0]["target"]["row_source_ordinal"] = json!(999);
        evidence_file.utf8_text = serde_json::to_string(&invalid).expect("invalid manifest");
        let error =
            normalize_virtual_project_source(source).expect_err("missing row must fail product");
        assert_eq!(error.code, "E_EVIDENCE_ROW_NOT_FOUND");
        assert_eq!(error.path.as_deref(), Some("evidence/budget.manifest.json"));
    }

    #[test]
    fn prepare_link_returns_linked_source_without_runtime_summary() {
        let prepared = build_project_prepare_link(sample_multifile_source(), "앱", true, true)
            .expect("prepare/link");
        assert_eq!(prepared.schema, "ddn.local_project.prepare_link.v1");
        assert_eq!(prepared.frontdoor, "wasm_project_prepare_link_json");
        assert_eq!(prepared.target_path, "app/main.ddn");
        // The exported name is now an authored D9 Definition, not a legacy binding.
        assert!(prepared.linked_execution_source.contains("속도:수 := 7."));
        assert!(prepared.linked_execution_source.contains("속도 보여주기."));
        assert!(!prepared.linked_execution_source_sha256.is_empty());
        assert!(prepared.symbol_boundary.output_rows.is_empty());
        assert!(prepared.state_hash_formula_change == false);
        assert!(!prepared.prepare_link_sha256.is_empty());
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["mode"],
            "oneshot"
        );
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["authority"],
            "parsed_program"
        );
    }

    #[test]
    fn authored_preparation_single_graph_keeps_metadata_maps_and_diagnostics() {
        let source=sample_multifile_source();
        let graph=build_project_graph(source.clone()).unwrap();
        let linked=build_linked_project_with_options(source.clone(),None,false,None).unwrap();
        let prepared=prepare_authored_execution(source.clone()).unwrap();
        assert_eq!(prepared.source,linked.execution_source);
        assert_eq!(prepared.target_path,graph.entry_path);
        assert_eq!(prepared.start_definition,graph.start_definition);
        let mut imports=BTreeSet::new();
        for symbol in &linked.symbol_boundary.resolved_symbols {
            for import in &linked.symbol_boundary.imports {
                if import.alias==symbol.alias && import.resolved_path==symbol.module_path {
                    imports.insert((import.importer_path.clone(),symbol.module_path.clone(),symbol.internal_name.clone()));
                }
            }
        }
        assert_eq!(prepared.imported_definitions,imports);
        for segment in &linked.execution_source_map {
            if segment.generated_start==segment.generated_end {continue;}
            let expected=Span::new(segment.source_boundaries[0],*segment.source_boundaries.last().unwrap());
            assert_eq!(prepared.original_span(Span::new(segment.generated_start,segment.generated_end)).unwrap(),
                (segment.source_path.clone(),expected));
        }
        for fault in 0..3 {
            let mut invalid=source.clone();
            match fault {
                0=>{let root=invalid.root_path.clone();invalid.files.retain(|file|file.path!=root);}
                1=>{invalid.files.push(invalid.files[0].clone());}
                _=>{invalid.files.iter_mut().find(|file|file.path=="app/main.ddn").unwrap()
                    .utf8_text="앱:움직씨 := { (".into();}
            }
            // Exact former two-pass path, not a source rewrite or new parser.
            let former=build_project_graph(invalid.clone()).and_then(|_|
                build_linked_project_with_options(invalid.clone(),None,false,None)).unwrap_err();
            let current=prepare_authored_execution(invalid).unwrap_err();
            assert_eq!(current.to_string(),former.to_string());
            assert_eq!(current.path,former.path);
            assert_eq!(current.source_span,former.source_span);
            assert_eq!(current.phase,former.phase);
            assert_eq!(current.offending_field,former.offending_field);
            assert_eq!(current.expected,former.expected);
            assert_eq!(current.actual,former.actual);
        }
    }

    #[test]
    fn module_sum_schema_is_a_definition_not_top_level_execution() {
        let mut source = sample_multifile_source();
        let main = source.files.iter_mut().find(|file| file.path == "app/main.ddn").unwrap();
        main.utf8_text = "선택:고름씨 := { $수(내용:수). }. 앱:움직씨 := { 자료 := 선택.$수(내용=4). 자료에 따라 { 선택.$수(내용)이면 { 내용 보여주기. }. }. }.".into();
        let summary = build_project_run_summary(source.clone(), "앱", false, false).unwrap();
        assert_eq!(summary.runtime_output_log_texts, vec!["4"]);
        let main = source.files.iter_mut().find(|file| file.path == "app/main.ddn").unwrap();
        main.utf8_text.push_str(" 선택:고름씨 := { $다른값. }.");
        // Same-file duplicate schemas are rejected by the existing parser
        // before the linker's cross-file namespace check is reached.
        assert_eq!(prepare_authored_execution(source).unwrap_err().code, "E_PROJECT_MODULE_PARSE");
    }

    #[test]
    fn module_sum_schema_does_not_allow_ordinary_top_level_execution() {
        let mut source = sample_multifile_source();
        let main = source.files.iter_mut().find(|file| file.path == "app/main.ddn").unwrap();
        main.utf8_text = "선택:고름씨 := { $수(내용:수). }. 4 보여주기. 앱:움직씨 := {}.".into();
        assert_eq!(prepare_authored_execution(source).unwrap_err().code, "E_PROJECT_TOP_LEVEL_EXECUTION");
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn prepare_link_and_project_run_bind_verified_dataset_identity_to_execution() {
        let source = sample_sealed_dataset_project();
        let symbols =
            build_project_symbol_boundary(source.clone()).expect("asset-aware symbol boundary");
        assert!(!symbols.symbol_boundary_sha256.is_empty());
        let prepared =
            build_project_prepare_link(source.clone(), "앱", true, false).expect("prepare dataset");
        assert_eq!(prepared.sealed_dataset_assets.len(), 1);
        let identity = &prepared.sealed_dataset_assets[0];
        assert_eq!(identity.artifact_id, "fixture.project-runtime");
        assert!(!identity.asset_manifest_sha256.is_empty());
        assert!(!identity.envelope_sha256.is_empty());
        assert!(!identity.payload_sha256.is_empty());
        assert_eq!(
            serde_json::to_value(&identity.source).expect("dataset source identity"),
            json!({
                "kind": "external_snapshot",
                "provider": "W1 fixture provider",
                "title": "project runtime fixture",
                "canonical_uri": "https://example.invalid/ddn/project-runtime",
                "published_or_version": "fixture-v1",
                "retrieved_at": "2026-07-27",
                "license_id": "LicenseRef-W1-fixture",
                "attribution": "W1 project runtime fixture"
            })
        );
        assert_eq!(
            prepared.runtime_execution_envelope["sealed_dataset_assets"][0]["content_sha256"],
            identity.content_sha256
        );

        let summary =
            build_project_run_summary(source.clone(), "앱", true, false).expect("run dataset");
        assert_eq!(
            summary.sealed_dataset_assets,
            prepared.sealed_dataset_assets
        );
        assert_eq!(
            summary.project_observation_trace["samples"][0]["cells"][0]["value_text"],
            "1월"
        );
        assert_eq!(
            summary.runtime_execution_envelope["sealed_dataset_assets"][0]["payload_sha256"],
            identity.payload_sha256
        );
        assert!(summary.runtime_state_hash.starts_with("blake3:"));

        let mut tampered = source;
        tampered
            .files
            .iter_mut()
            .find(|file| file.path == "datasets/project-runtime/payload.csv")
            .expect("payload")
            .utf8_text
            .push('x');
        let error = build_project_prepare_link(tampered, "앱", true, false).unwrap_err();
        assert_eq!(error.code, "E_PROJECT_ASSET_REGISTRY");
        assert!(error.message.contains("RESOURCE_HASH_MISMATCH"));
    }

    #[test]
    fn linked_multifile_tick_hook_uses_parsed_live_execution_mode() {
        let mut source = sample_multifile_source();
        let entry = source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry file");
        entry.utf8_text =
            "(운동=운동) 쓰임.\n앱:움직씨 := { (매마디)마다 {\n  운동.속도 보여주기.\n}. }.".to_string();
        let prepared = build_project_prepare_link(source, "앱", true, true).expect("prepare/link");
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["mode"],
            "live"
        );
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["live_hook_count"],
            1
        );
    }

    #[test]
    fn linked_multifile_boim_preserves_alias_meaning_and_successor_trace() {
        let source = sample_boim_multifile_source();
        let raw_entry = source
            .files
            .iter()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry file")
            .utf8_text
            .clone();
        let raw_error =
            runtime_surface::run_summary_from_source(&raw_entry, "app/main.ddn", Some(1))
                .expect_err("raw entry must not bypass the project linker");
        assert!(
            raw_error.contains("E_RUNTIME_UNSUPPORTED_STMT")
                || raw_error.contains("E_RUNTIME_VAR")
                || raw_error.contains("파싱 실패"),
            "unexpected raw-entry refusal: {raw_error}"
        );

        let prepared =
            build_project_prepare_link(source.clone(), "앱", true, false).expect("prepare/link");
        assert_eq!(prepared.target_path, "app/main.ddn");
        assert_eq!(
            prepared.runtime_execution_envelope["execution_mode_decision"]["mode"],
            "live"
        );
        assert!(prepared.linked_execution_source.contains("시작값 := 20."));
        assert!(prepared.linked_execution_source.contains("증가값 := 3."));
        assert!(
            prepared.linked_execution_source.contains("증가량: 증가값"),
            "{}",
            prepared.linked_execution_source
        );
        assert!(prepared.linked_execution_source.contains(") 내보임."));
        assert!(!prepared.linked_execution_source.contains("도움.증가값"));

        let summary =
            build_project_run_summary(source, "앱", true, false).expect("linked project run");
        assert_eq!(
            summary.project_observation_trace["schema"],
            "ddn.public_observation_trace.v1"
        );
        assert!(summary
            .project_observation_trace_hash
            .starts_with("sha256:"));
        assert_eq!(
            summary.project_observation_trace["columns"]
                .as_array()
                .expect("columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["경과", "각도", "증가량"]
        );
        assert_eq!(
            summary.project_observation_trace["samples"][0]["cells"]
                .as_array()
                .expect("cells")
                .iter()
                .map(|cell| cell["value_text"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["1@s", "23@rad", "3"]
        );
        assert_eq!(summary.output_rows.len(), 1);
        assert_eq!(summary.runtime_output_log_texts, Vec::<String>::new());
        assert_eq!(
            summary.work_cost_receipt.counters.sealed_input_bytes,
            summary.linked_execution_source.len() as u64
        );
        assert_eq!(
            summary.work_cost_receipt.counters.result_artifact_bytes,
            serde_json::to_vec(&summary)
                .expect("project summary bytes")
                .len() as u64
        );
        assert!(!summary.work_cost_receipt.slice_limits_enforced);
        assert!(!summary.work_cost_receipt.rd206_resource_capability);
        assert_eq!(
            summary.output_rows[0]["cells"]
                .as_array()
                .expect("output cells")
                .iter()
                .map(|cell| cell["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["경과", "각도", "증가량"]
        );
    }

    #[test]
    fn rename_edit_plan_updates_only_structural_references() {
        let mut source = sample_multifile_source();
        let main = source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .unwrap();
        main.utf8_text =
            "(운동=운동) 쓰임.\n앱:움직씨 := { \"./lib/운동\" 보여주기. }.\n# ./lib/운동은 설명 문자열"
                .to_string();
        let before_hash = virtual_project_source_sha256(&source).expect("hash");
        let plan = build_project_edit_plan(ProjectEditPlanRequest {
            schema: PROJECT_EDIT_PLAN_SCHEMA.to_string(),
            source,
            operation: ProjectEditOperation::RenameFile {
                from_path: "lib/운동.ddn".to_string(),
                to_path: "lib/힘.ddn".to_string(),
            },
            expected_source_sha256: before_hash.clone(),
        })
        .expect("rename plan");
        assert_eq!(plan.before_source_sha256, before_hash);
        assert!(plan.atomic_apply);
        assert!(plan.lock_stale);
        assert!(plan
            .project_source
            .files
            .iter()
            .any(|file| file.path == "lib/힘.ddn"));
        let main = plan
            .project_source
            .files
            .iter()
            .find(|file| file.path == "app/main.ddn")
            .unwrap();
        assert!(main.utf8_text.contains("(운동=운동) 쓰임."));
        assert!(main.utf8_text.contains("\"./lib/운동\" 보여주기."));
        assert!(main.utf8_text.contains("# ./lib/운동은 설명 문자열"));
        let root = plan
            .project_source
            .files
            .iter()
            .find(|file| file.path == PROJECT_ROOT_FILE)
            .unwrap();
        assert!(root.utf8_text.contains("머리=\"lib/힘.ddn\""));
        assert!(root.utf8_text.contains("원문=\"lib/힘.ddn\""));
        assert!(!root.utf8_text.contains("lib/운동.ddn"));
    }

    #[test]
    fn rename_edit_plan_rejects_stale_hash_without_change() {
        let source = sample_multifile_source();
        let original = serde_json::to_string(&source).unwrap();
        let error = build_project_edit_plan(ProjectEditPlanRequest {
            schema: PROJECT_EDIT_PLAN_SCHEMA.to_string(),
            source: source.clone(),
            operation: ProjectEditOperation::RenameFile {
                from_path: "lib/운동.ddn".to_string(),
                to_path: "lib/힘.ddn".to_string(),
            },
            expected_source_sha256: "stale".to_string(),
        })
        .unwrap_err();
        assert_eq!(error.code, "E_PROJECT_EDIT_SOURCE_CONFLICT");
        assert_eq!(serde_json::to_string(&source).unwrap(), original);
    }

    #[test]
    fn rename_edit_plan_rejects_root_and_duplicate_target() {
        let source = sample_multifile_source();
        let hash = virtual_project_source_sha256(&source).unwrap();
        let root_error = build_project_edit_plan(ProjectEditPlanRequest {
            schema: PROJECT_EDIT_PLAN_SCHEMA.to_string(),
            source: source.clone(),
            operation: ProjectEditOperation::RenameFile {
                from_path: PROJECT_ROOT_FILE.to_string(),
                to_path: "root.ddn".to_string(),
            },
            expected_source_sha256: hash.clone(),
        })
        .unwrap_err();
        assert_eq!(root_error.code, "E_PROJECT_ROOT_RENAME_FORBIDDEN");
        let duplicate_error = build_project_edit_plan(ProjectEditPlanRequest {
            schema: PROJECT_EDIT_PLAN_SCHEMA.to_string(),
            source,
            operation: ProjectEditOperation::RenameFile {
                from_path: "lib/운동.ddn".to_string(),
                to_path: "app/main.ddn".to_string(),
            },
            expected_source_sha256: hash,
        })
        .unwrap_err();
        assert_eq!(duplicate_error.code, "E_PROJECT_PATH_DUPLICATE");
    }

    fn edit_request(
        source: VirtualProjectSource,
        operation: ProjectEditOperation,
    ) -> ProjectEditPlanRequest {
        let expected_source_sha256 = virtual_project_source_sha256(&source).unwrap();
        ProjectEditPlanRequest {
            schema: PROJECT_EDIT_PLAN_SCHEMA.to_string(),
            source,
            operation,
            expected_source_sha256,
        }
    }

    #[test]
    fn structural_module_add_rejects_duplicate_names_injection_and_root_escape() {
        for (path, name) in [
            ("lib/새.ddn", "앱"),
            ("lib/새.ddn", "\"새\""),
            ("lib/새.ddn", "새, 머리=\"lib/새.ddn\""),
        ] {
            let source = sample_multifile_source();
            let before = virtual_project_source_sha256(&source).unwrap();
            assert!(build_project_edit_plan(edit_request(source.clone(),
                ProjectEditOperation::AddModule { path: path.into(), module_name: name.into() })).is_err());
            assert_eq!(virtual_project_source_sha256(&source).unwrap(), before);
        }
        let mut source = sample_multifile_source();
        source.root_path = "work/뿌리.ddn".into();
        source.entry_path = "work/app/main.ddn".into();
        for file in &mut source.files { file.path = format!("work/{}", file.path); }
        let error = build_project_edit_plan(edit_request(source,
            ProjectEditOperation::AddModule { path: "outside.ddn".into(), module_name: "밖".into() }))
            .expect_err("cannot add a module outside the root directory");
        assert_eq!(error.code, "E_PROJECT_PATH_ESCAPE");
    }

    #[test]
    fn structural_edit_adds_declared_module_and_rejects_implicit_membership_and_start() {
        let source = sample_multifile_source();
        let added = build_project_edit_plan(edit_request(
            source,
            ProjectEditOperation::AddModule {
                path: "lib/힘.ddn".to_string(),
                module_name: "힘".to_string(),
            },
        ))
        .expect("add module");
        assert_eq!(added.operation, "add_module");
        assert!(added.atomic_apply);
        assert_eq!(added.patches.len(), 2);
        assert!(added.project_source.files.iter().any(|file| {
            file.path == "lib/힘.ddn"
                && file.utf8_text.contains("(값=값) 드러냄.")
                && file.utf8_text.contains("값:수 := 0.")
                && !file.utf8_text.contains("드러냄 {")
                && !file.utf8_text.contains("채비 {")
        }));

        let generated_texts = added
            .project_source
            .files
            .iter()
            .map(|file| (file.path.clone(), file.utf8_text.clone()))
            .collect::<BTreeMap<_, _>>();
        let generated_directives = current_module_directives(
            generated_texts
                .get("lib/힘.ddn")
                .expect("generated module text"),
            "lib/힘.ddn",
            &generated_texts,
        )
        .expect("generated current module directives")
        .expect("generated current module directive surface");
        assert_eq!(
            generated_directives
                .exports
                .iter()
                .map(|entry| (entry.external_name.as_str(), entry.internal_name.as_str()))
                .collect::<Vec<_>>(),
            vec![("값", "값")]
        );

        let mut cross_module = added.project_source.clone();
        cross_module
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry file")
            .utf8_text = "(힘=힘) 쓰임.\n앱:움직씨 := { 힘.값 보여주기. }.\n".to_string();
        let boundary = build_linked_project_with_options(cross_module, None, false, None)
            .expect("generated current module must resolve through the local-project linker")
            .symbol_boundary;
        assert!(boundary.exports.iter().any(|export| {
            export.module_path == "lib/힘.ddn"
                && export.external_name == "값"
                && export.internal_name == "값"
        }));
        assert!(boundary.resolved_symbols.iter().any(|symbol| {
            symbol.alias == "힘"
                && symbol.symbol == "값"
                && symbol.module_path == "lib/힘.ddn"
                && symbol.internal_name == "값"
        }));

        // D9: a copied file does not become a module without an authored
        // declaration, and a file path does not identify an executable definition.
        let duplicated = build_project_edit_plan(edit_request(
            added.project_source.clone(),
            ProjectEditOperation::DuplicateFile {
                from_path: "lib/힘.ddn".to_string(),
                to_path: "lib/힘_복사.ddn".to_string(),
            },
        ))
        .expect_err("undeclared duplicate must not silently enter the module closure");
        assert_eq!(duplicated.code, "E_PROJECT_UNDECLARED_SOURCE");

        let entry = build_project_edit_plan(edit_request(
            added.project_source,
            ProjectEditOperation::SetEntry {
                path: "lib/힘.ddn".to_string(),
            },
        ))
        .expect_err("path-only start must not infer a definition");
        assert_eq!(entry.code, "E_PROJECT_START_REFERENCE_REQUIRED");
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn sealed_dataset_binding_edit_uses_verified_handle_and_real_runtime_call() {
        let mut source = sample_sealed_dataset_project();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "app/main.ddn")
            .expect("entry")
            .utf8_text = "앱:움직씨 := {\n  값:수 := 1.\n  값 보여주기.\n}\n".to_string();
        let handle = project_asset_registry_from_source(&source)
            .expect("asset registry")
            .expect("asset registry present")
            .sealed_dataset_identities()
            .expect("sealed identities")[0]
            .handle
            .clone();
        let plan = build_project_edit_plan(edit_request(
            source.clone(),
            ProjectEditOperation::BindSealedDataset {
                binding_name: "자료값".to_string(),
                dataset_handle: handle.clone(),
            },
        ))
        .expect("verified dataset binding plan");
        assert_eq!(plan.operation, "bind_sealed_dataset");
        assert!(plan.atomic_apply);
        assert_eq!(plan.affected_symbols, vec!["자료값"]);
        let entry = plan
            .project_source
            .files
            .iter()
            .find(|file| file.path == "app/main.ddn")
            .expect("bound entry");
        let binding = "자료값:묶음 := (@\"datasets/project-runtime/envelope.json\") 자료.열기.";
        assert!(entry.utf8_text.contains(binding));
        assert!(
            entry.utf8_text.find(binding).expect("binding")
                < entry
                    .utf8_text
                    .find("값:수 := 1.")
                    .expect("current consumer")
        );
        let summary =
            build_project_run_summary(plan.project_source, "앱", true, false).expect("project run");
        assert_eq!(summary.sealed_dataset_assets.len(), 1);
        assert_eq!(summary.sealed_dataset_assets[0].handle, handle);
        assert_eq!(summary.output_rows.len(), 1);

        let unverified = build_project_edit_plan(edit_request(
            source,
            ProjectEditOperation::BindSealedDataset {
                binding_name: "다른자료".to_string(),
                dataset_handle: "0000000000000000".to_string(),
            },
        ))
        .expect_err("unverified handle must fail closed");
        assert_eq!(unverified.code, "E_PROJECT_DATASET_HANDLE_UNVERIFIED");
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn execution_recipe_identity_binds_prepare_link_and_setup_without_state_hash_changes() {
        let prepared =
            build_project_prepare_link(sample_sealed_dataset_project(), "앱", true, false)
                .expect("prepare sealed project");
        let first = build_project_execution_recipe(ProjectExecutionRecipeRequest {
            schema: PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA.to_string(),
            prepare_link_sha256: prepared.prepare_link_sha256.clone(),
            recipe_version: 1,
            setup: json!({"seed": "w1", "ticks": 1}),
        })
        .expect("execution recipe");
        let reordered = build_project_execution_recipe(ProjectExecutionRecipeRequest {
            schema: PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA.to_string(),
            prepare_link_sha256: prepared.prepare_link_sha256.clone(),
            recipe_version: 1,
            setup: json!({"ticks": 1, "seed": "w1"}),
        })
        .expect("reordered execution recipe");
        assert_eq!(
            first.execution_recipe_sha256,
            reordered.execution_recipe_sha256
        );
        let changed_setup = build_project_execution_recipe(ProjectExecutionRecipeRequest {
            schema: PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA.to_string(),
            prepare_link_sha256: prepared.prepare_link_sha256.clone(),
            recipe_version: 1,
            setup: json!({"seed": "w1", "ticks": 2}),
        })
        .expect("changed setup");
        assert_ne!(
            first.execution_recipe_sha256,
            changed_setup.execution_recipe_sha256
        );
        let changed_prepare = build_project_execution_recipe(ProjectExecutionRecipeRequest {
            schema: PROJECT_EXECUTION_RECIPE_REQUEST_SCHEMA.to_string(),
            prepare_link_sha256: "a".repeat(64),
            recipe_version: 1,
            setup: json!({"seed": "w1", "ticks": 1}),
        })
        .expect("changed prepare");
        assert_ne!(
            first.execution_recipe_sha256,
            changed_prepare.execution_recipe_sha256
        );
        assert!(!prepared.state_hash_formula_change);
    }

    #[test]
    fn delete_edit_plan_rejects_referenced_module_without_mutation() {
        let source = sample_multifile_source();
        let original = serde_json::to_string(&source).unwrap();
        let error = build_project_edit_plan(edit_request(
            source.clone(),
            ProjectEditOperation::DeleteFile {
                path: "lib/운동.ddn".to_string(),
            },
        ))
        .unwrap_err();
        assert_eq!(error.code, "E_PROJECT_FILE_REFERENCED");
        assert_eq!(serde_json::to_string(&source).unwrap(), original);
    }

    #[test]
    fn delete_edit_plan_removes_unreferenced_file_atomically() {
        let mut source = sample_multifile_source();
        source.files.push(VirtualProjectFileSource {
            path: "lib/안씀.ddn".to_string(),
            role: "ddn_source".to_string(),
            utf8_text: "드러냄 { 값. }\n값 <- 1.".to_string(),
        });
        let plan = build_project_edit_plan(edit_request(
            source,
            ProjectEditOperation::DeleteFile {
                path: "lib/안씀.ddn".to_string(),
            },
        ))
        .expect("delete");
        assert!(plan.atomic_apply);
        assert!(!plan
            .project_source
            .files
            .iter()
            .any(|file| file.path == "lib/안씀.ddn"));
    }

    #[test]
    fn alias_show_token_fallback_does_not_cross_statement_line() {
        let requests = collect_alias_show_requests_from_tokens(
            "속도 <- 속도 - 9.8 * dt.\n위치 보여주기.\n힘.중력가속도 보여주기.",
        )
        .expect("token requests");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].alias, "힘");
        assert_eq!(requests[0].symbol, "중력가속도");
    }

    fn extraction_request(source: VirtualProjectSource) -> GajiExtractionPlanRequest {
        GajiExtractionPlanRequest {
            schema: GAJI_EXTRACTION_PLAN_SCHEMA.to_string(),
            source,
            selected_modules: vec!["lib/운동.ddn".to_string()],
            gaji_id: "운동".to_string(),
            version: "1.0.0".to_string(),
            description: "운동 helper".to_string(),
            tags: vec!["물리".to_string()],
            profile: "내".to_string(),
            origin_project_ref: Some("project:test".to_string()),
            origin_gaji: None,
            relation: "extracted".to_string(),
        }
    }

    #[test]
    fn gaji_extraction_plan_uses_structural_exports_and_dependency_closure() {
        let mut source = sample_multifile_source();
        source.files.iter_mut().find(|file| file.path == "뿌리.ddn").unwrap().utf8_text =
            "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\"),(이름=운동,머리=\"lib/운동.ddn\",원문=\"lib/운동.ddn\"),(이름=수치,머리=\"lib/수치.ddn\",원문=\"lib/수치.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".into();
        source.files.iter_mut().find(|file| file.path == "lib/운동.ddn").unwrap().utf8_text =
            "(수치=수치) 쓰임.\n(속도=속도) 드러냄.\n// 드러냄 { 가짜. }\n설명 := \"드러냄 { 문자열가짜. }\".\n속도 := 수치.기본값.".to_string();
        source.files.push(VirtualProjectFileSource {
            path: "lib/수치.ddn".to_string(),
            role: "ddn_source".to_string(),
            utf8_text: "(기본값=기본값) 드러냄.\n기본값 := 7.".to_string(),
        });
        let first = build_gaji_extraction_plan(extraction_request(source.clone())).expect("plan");
        let second = build_gaji_extraction_plan(extraction_request(source)).expect("plan repeat");
        assert_eq!(first.included_modules, vec!["lib/수치.ddn", "lib/운동.ddn"]);
        assert_eq!(first.public_symbols.len(), 2);
        assert!(!first
            .public_symbols
            .iter()
            .any(|item| item.external_name.contains("가짜")));
        assert_eq!(first.content_sha256, second.content_sha256);
        assert!(first.publishable);
    }

    #[test]
    fn gaji_extraction_plan_rejects_private_only_module() {
        let mut source = sample_multifile_source();
        source
            .files
            .iter_mut()
            .find(|file| file.path == "lib/운동.ddn")
            .unwrap()
            .utf8_text = "비공개값 <- 7.".to_string();
        let error = build_gaji_extraction_plan(extraction_request(source)).unwrap_err();
        assert_eq!(error.code, "E_GAJI_PUBLIC_API_EMPTY");
    }

    #[test]
    fn local_gaji_artifact_is_deterministic_and_rejects_tamper() {
        let first = build_local_gaji_artifact(extraction_request(sample_multifile_source()))
            .expect("artifact");
        let second = build_local_gaji_artifact(extraction_request(sample_multifile_source()))
            .expect("artifact repeat");
        assert_eq!(first.package_sha256, second.package_sha256);
        assert_eq!(first.manifest_path, "gaji.toml");
        assert!(first
            .files
            .iter()
            .any(|file| file.path == "ddn/lib/운동.ddn"));
        let mut tampered = first;
        let source = tampered
            .files
            .iter_mut()
            .find(|file| file.path.starts_with("ddn/"))
            .unwrap();
        source.utf8_text.push_str("\n변조 <- 1.");
        assert_eq!(
            verify_local_gaji_artifact(&tampered).unwrap_err().code,
            "E_GAJI_ARTIFACT_FILE_HASH"
        );
    }

    #[test]
    fn local_gaji_remix_preserves_origin_and_requires_explicit_lineage() {
        let original =
            build_local_gaji_artifact(extraction_request(sample_multifile_source())).unwrap();
        let mut remix_request = extraction_request(sample_multifile_source());
        remix_request.gaji_id = "운동-내버전".to_string();
        remix_request.origin_gaji = Some(format!("{}@{}", original.gaji_id, original.version));
        remix_request.relation = "remix".to_string();
        let remix = build_local_gaji_artifact(remix_request).unwrap();
        assert_eq!(remix.provenance.origin_gaji.as_deref(), Some("운동@1.0.0"));
        assert_eq!(remix.provenance.relation, "remix");
        assert_eq!(original.provenance.origin_gaji, None);
        let mut missing = extraction_request(sample_multifile_source());
        missing.relation = "remix".to_string();
        assert_eq!(
            build_local_gaji_artifact(missing).unwrap_err().code,
            "E_GAJI_REMIX_ORIGIN_REQUIRED"
        );
    }

    #[test]
    fn local_gaji_artifact_enforces_path_hash_and_size_bounds() {
        let base =
            build_local_gaji_artifact(extraction_request(sample_multifile_source())).unwrap();
        let mut manifest_mismatch = base.clone();
        manifest_mismatch.content_sha256 = "0".repeat(64);
        assert_eq!(
            verify_local_gaji_artifact(&manifest_mismatch)
                .unwrap_err()
                .code,
            "E_GAJI_MANIFEST_CONTENT_HASH"
        );

        let mut case_duplicate = base.clone();
        let mut duplicate = case_duplicate
            .files
            .iter()
            .find(|file| file.path.starts_with("ddn/"))
            .unwrap()
            .clone();
        duplicate.path = duplicate.path.to_uppercase();
        case_duplicate.files.push(duplicate);
        case_duplicate.package_sha256 = local_gaji_artifact_package_hash(&case_duplicate.files);
        assert_eq!(
            verify_local_gaji_artifact(&case_duplicate)
                .unwrap_err()
                .code,
            "E_GAJI_ARTIFACT_PATH_DUPLICATE"
        );

        let mut oversized = base.clone();
        oversized.files.push(local_gaji_artifact_file(
            "ddn/lib/큰파일.ddn",
            "가".repeat(GAJI_ARTIFACT_MAX_SINGLE_FILE_BYTES),
        ));
        oversized.package_sha256 = local_gaji_artifact_package_hash(&oversized.files);
        assert_eq!(
            verify_local_gaji_artifact(&oversized).unwrap_err().code,
            "E_GAJI_ARTIFACT_FILE_BOUND"
        );

        let mut too_many = base;
        while too_many.files.len() <= GAJI_ARTIFACT_MAX_FILES {
            let index = too_many.files.len();
            too_many.files.push(local_gaji_artifact_file(
                &format!("ddn/lib/파일{index}.ddn"),
                "값 <- 1.".to_string(),
            ));
        }
        too_many.package_sha256 = local_gaji_artifact_package_hash(&too_many.files);
        assert_eq!(
            verify_local_gaji_artifact(&too_many).unwrap_err().code,
            "E_GAJI_ARTIFACT_FILE_COUNT_BOUND"
        );
    }

    #[test]
    fn local_gotgan_publish_is_immutable_and_searches_public_metadata() {
        let artifact = build_local_gaji_artifact(extraction_request(sample_multifile_source()))
            .expect("artifact");
        let empty = LocalGotganStore {
            schema: LOCAL_GOTGAN_STORE_SCHEMA.to_string(),
            artifacts: Vec::new(),
        };
        let store = publish_local_gotgan(empty, artifact.clone()).expect("publish");
        let repeated =
            publish_local_gotgan(store.clone(), artifact.clone()).expect("idempotent same bytes");
        assert_eq!(repeated.artifacts.len(), 1);
        let rows = search_local_gotgan(&store, "속도", &["내".to_string()]).expect("search");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].gaji_id, "운동");
        let mut conflicting = artifact;
        let file = conflicting
            .files
            .iter_mut()
            .find(|file| file.path.starts_with("ddn/"))
            .unwrap();
        file.utf8_text.push_str("\n다른값 <- 1.");
        file.byte_length = file.utf8_text.len();
        file.sha256 = sha256_hex(file.utf8_text.as_bytes());
        conflicting.package_sha256 = local_gaji_artifact_package_hash(&conflicting.files);
        assert_eq!(
            publish_local_gotgan(store, conflicting).unwrap_err().code,
            "E_GOTGAN_VERSION_IMMUTABLE"
        );
    }

    #[test]
    fn local_gotgan_search_orders_one_hundred_artifacts_deterministically() {
        let mut store = LocalGotganStore {
            schema: LOCAL_GOTGAN_STORE_SCHEMA.to_string(),
            artifacts: Vec::new(),
        };
        for index in (0..105).rev() {
            let mut request = extraction_request(sample_multifile_source());
            request.gaji_id = format!("도구-{index:03}");
            request.description = "결정적 검색 도구".to_string();
            store =
                publish_local_gotgan(store, build_local_gaji_artifact(request).unwrap()).unwrap();
        }
        let first = search_local_gotgan(&store, "검색", &["내".to_string()]).unwrap();
        let second = search_local_gotgan(&store, "검색", &["내".to_string()]).unwrap();
        assert_eq!(first.len(), 105);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert_eq!(first.first().unwrap().gaji_id, "도구-000");
        assert_eq!(first.last().unwrap().gaji_id, "도구-104");
    }

    fn lifecycle_request(
        source: VirtualProjectSource,
        operation: GajiLifecycleOperation,
    ) -> GajiLifecyclePlanRequest {
        GajiLifecyclePlanRequest {
            schema: GAJI_LIFECYCLE_PLAN_SCHEMA.to_string(),
            expected_source_sha256: virtual_project_source_sha256(&source).unwrap(),
            expected_lock_sha256: None,
            source,
            operation,
        }
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn gaji_lifecycle_installs_with_lock_and_removes_unused_atomically() {
        let artifact =
            build_local_gaji_artifact(extraction_request(sample_multifile_source())).unwrap();
        let installed = build_gaji_lifecycle_plan(lifecycle_request(
            sample_multifile_source(),
            GajiLifecycleOperation::Install {
                alias: "힘".to_string(),
                artifact,
            },
        ))
        .expect("install");
        assert!(installed.atomic_apply);
        assert!(installed.after_lock_sha256.is_some());
        assert!(installed
            .project_source
            .files
            .iter()
            .any(|file| file.path == "vendor/운동/gaji.toml"));
        build_project_gaji_lock(installed.project_source.clone(), true)
            .expect("installed lock verifies");
        let removed = build_gaji_lifecycle_plan(lifecycle_request(
            installed.project_source,
            GajiLifecycleOperation::Remove {
                alias: "힘".to_string(),
            },
        ))
        .expect("remove unused");
        assert!(removed.after_lock_sha256.is_none());
        assert!(!removed
            .project_source
            .files
            .iter()
            .any(|file| file.path.starts_with("vendor/운동/")));
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn gaji_lifecycle_rejects_stale_source_and_referenced_remove_without_mutation() {
        let source = sample_multifile_source();
        let artifact = build_local_gaji_artifact(extraction_request(source.clone())).unwrap();
        let mut stale = lifecycle_request(
            source.clone(),
            GajiLifecycleOperation::Install {
                alias: "힘".to_string(),
                artifact: artifact.clone(),
            },
        );
        stale.expected_source_sha256 = "0".repeat(64);
        assert_eq!(
            build_gaji_lifecycle_plan(stale).unwrap_err().code,
            "E_GAJI_LIFECYCLE_SOURCE_STALE"
        );
        let installed = build_gaji_lifecycle_plan(lifecycle_request(
            source,
            GajiLifecycleOperation::Install {
                alias: "운동".to_string(),
                artifact,
            },
        ))
        .unwrap();
        let before = serde_json::to_string(&installed.project_source).unwrap();
        let error = build_gaji_lifecycle_plan(lifecycle_request(
            installed.project_source.clone(),
            GajiLifecycleOperation::Remove {
                alias: "운동".to_string(),
            },
        ))
        .unwrap_err();
        assert_eq!(error.code, "E_GAJI_REMOVE_REFERENCED");
        assert_eq!(
            serde_json::to_string(&installed.project_source).unwrap(),
            before
        );
    }

    #[test]
    #[cfg(feature = "beta-project-features")]
    fn gaji_lifecycle_updates_with_diff_and_rolls_back_to_retained_bytes() {
        let source = sample_multifile_source();
        let artifact_v1 = build_local_gaji_artifact(extraction_request(source.clone())).unwrap();
        let installed = build_gaji_lifecycle_plan(lifecycle_request(
            source.clone(),
            GajiLifecycleOperation::Install {
                alias: "힘".to_string(),
                artifact: artifact_v1.clone(),
            },
        ))
        .unwrap();
        let v1_lock = installed.after_lock_sha256.clone();

        let mut changed_source = source;
        changed_source
            .files
            .iter_mut()
            .find(|file| file.path == "lib/운동.ddn")
            .unwrap()
            .utf8_text = "드러냄 { 추가힘. }\n추가힘 <- 2.".to_string();
        let mut request_v2 = extraction_request(changed_source);
        request_v2.version = "2.0.0".to_string();
        let artifact_v2 = build_local_gaji_artifact(request_v2).unwrap();
        let updated = build_gaji_lifecycle_plan(lifecycle_request(
            installed.project_source,
            GajiLifecycleOperation::Update {
                alias: "힘".to_string(),
                artifact: artifact_v2,
            },
        ))
        .unwrap();
        let diff = updated.version_diff.as_ref().expect("version diff");
        assert_eq!(diff.from_version.as_deref(), Some("1.0.0"));
        assert_eq!(diff.to_version.as_deref(), Some("2.0.0"));
        assert!(diff
            .changed_files
            .iter()
            .any(|path| path == "ddn/lib/운동.ddn"));
        assert!(diff
            .added_public_symbols
            .iter()
            .any(|symbol| symbol.ends_with(":추가힘")));
        assert!(diff
            .removed_public_symbols
            .iter()
            .any(|symbol| symbol.ends_with(":속도")));
        assert!(diff.risky);

        let rolled_back = build_gaji_lifecycle_plan(lifecycle_request(
            updated.project_source,
            GajiLifecycleOperation::Update {
                alias: "힘".to_string(),
                artifact: artifact_v1,
            },
        ))
        .unwrap();
        assert_eq!(rolled_back.after_lock_sha256, v1_lock);
        let rollback_diff = rolled_back.version_diff.as_ref().unwrap();
        assert_eq!(rollback_diff.from_version.as_deref(), Some("2.0.0"));
        assert_eq!(rollback_diff.to_version.as_deref(), Some("1.0.0"));
        assert!(rollback_diff.risky);
    }

    #[test]
    fn rejects_traversal_and_backslash() {
        assert_eq!(
            normalize_project_path("../x").unwrap_err().code,
            "E_PROJECT_PATH_SEGMENT"
        );
        assert_eq!(
            normalize_project_path("lib\\x.ddn").unwrap_err().code,
            "E_PROJECT_PATH_BACKSLASH"
        );
    }

    #[test]
    fn missing_module_diagnostic_points_to_importing_alias() {
        let mut source = sample_multifile_source();
        source.files.retain(|file| file.path != "lib/운동.ddn");
        source.files.iter_mut().find(|file| file.path == "뿌리.ddn").unwrap().utf8_text =
            "(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"app/main.ddn\",원문=\"app/main.ddn\")], 시작=(모듈=앱,정의=앱)) 얽음.".into();
        let entry_text = source.files.iter().find(|file| file.path == "app/main.ddn")
            .unwrap().utf8_text.clone();
        let error = build_project_symbol_boundary(source).unwrap_err();
        assert_eq!(error.code, "E_MODULE_MISSING");
        assert_eq!(error.path.as_deref(), Some("app/main.ddn"));
        let (start, end) = error.source_span.expect("import alias span");
        assert_eq!(
            &entry_text[start..end],
            "운동"
        );
        let json = error.to_json_value();
        assert_eq!(json["source_ref"], "app/main.ddn");
        assert_eq!(json["severity"], "fatal");
    }

    #[test]
    fn project_block_detection_ignores_title_strings_and_comments() {
        assert!(!contains_project_block(
            "설정 { 제목: \"내 프로젝트\". }.\n# 프로젝트 { 가짜. }"
        ));
        assert!(contains_project_block("프로젝트 { 이름: \"정본\". }."));
    }

    #[test]
    fn discovers_only_explicit_virtual_root_without_project_json_fallback() {
        let request = ProjectRootDiscoveryRequest {
            schema: PROJECT_ROOT_DISCOVERY_REQUEST_SCHEMA.to_string(),
            start_path: "chosen.ddn".to_string(),
            explicit_project_path: None,
            files: vec![
                ProjectRootDiscoveryFile {
                    path: "chosen.ddn".to_string(),
                    utf8_text: Some("(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")]) 얽음.".into()),
                },
                ProjectRootDiscoveryFile {
                    path: PROJECT_JSON_FILE.to_string(),
                    utf8_text: None,
                },
                ProjectRootDiscoveryFile {
                    path: "app/main.ddn".to_string(),
                    utf8_text: None,
                },
            ],
        };
        let result = discover_virtual_project_root(request).expect("discover");
        assert_eq!(result.root_kind, "explicit_work_root");
        assert_eq!(result.root_marker_path.as_deref(), Some("chosen.ddn"));
        assert!(!result.used_fallback);
        assert!(result.project_json_path.is_none());
    }

    #[test]
    fn virtual_root_discovery_rejects_legacy_only() {
        let request = ProjectRootDiscoveryRequest {
            schema: PROJECT_ROOT_DISCOVERY_REQUEST_SCHEMA.to_string(),
            start_path: "app/main.ddn".to_string(),
            explicit_project_path: None,
            files: vec![
                ProjectRootDiscoveryFile {
                    path: "프로젝트.ddn".to_string(),
                    utf8_text: None,
                },
                ProjectRootDiscoveryFile {
                    path: "app/main.ddn".to_string(),
                    utf8_text: None,
                },
            ],
        };
        assert_eq!(
            discover_virtual_project_root(request).unwrap_err().code,
            // A3/RD526: reject absent explicit root before examining filenames.
            "E_PROJECT_ROOT_EXPLICIT_REQUIRED"
        );
    }
