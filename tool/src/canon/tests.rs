//! Existing tests; namespace and included source bytes preserved.
    use super::{
        apply_maegim_source_edit, canonicalize, extract_prompt_string_literals,
        has_exec_policy_surface, maegim_control_plan_with_source,
    };

    #[test]
    fn wo03_explicit_current_condition_canon_keeps_legacy_projection_separate() {
        // The epoch-less projection in this module remains a legacy adapter.
        // Current source must use the explicit shared profile, not content inference.
        let source = "{ 참 }이면 { 1 보여주기. } 아니면 { { 거짓 }이면 { 2 보여주기. }. }.";
        let current = crate::runtime_surface::canonicalize_shared_source_with_supported_profile(
            source, Some("v1-core-v25"),
        ).expect("explicit current conditional");
        assert!(current.ddn.contains("}이면"));
        assert!(!current.ddn.contains("만약"));
        let twice = crate::runtime_surface::canonicalize_shared_source_with_supported_profile(
            &current.ddn, Some("v1-core-v25"),
        ).expect("current canon roundtrip");
        assert_eq!(current.ddn, twice.ddn);
        let legacy = canonicalize("만약 참 이라면 { 1 보여주기. }.", false)
            .expect("legacy projection remains available");
        assert!(legacy.ddn.contains("만약 참 이라면"));
        assert!(crate::runtime_surface::canonicalize_shared_source_with_supported_profile(
            &legacy.ddn, Some("v1-core-v25"),
        ).is_err(), "legacy surface must not become a current alias");
    }

    #[test]
    fn prompt_extraction_uses_lexer_comment_and_string_boundaries() {
        let source = "// ??(\"가짜\")\n메모 <- \"??(\\\"문자열\\\")\".\n답 <- ??(\"진짜\").\n";
        assert_eq!(
            extract_prompt_string_literals(source).expect("extract prompts"),
            vec!["진짜"]
        );
    }

    #[test]
    fn prompt_extraction_rejects_unterminated_string() {
        let error = extract_prompt_string_literals("답 <- ??(\"미완료).\n")
            .expect_err("unterminated prompt must fail closed");
        assert_eq!(error.code(), "E_CANON_UNTERM_STRING");
    }

    #[test]
    fn legacy_projection_canon_rejects_untyped_rendezvous_ownership() {
        let source = r#"
(온도:수, 풍속:수) 기상특보:알림씨 = {
}.
기상청:임자 = {
  (온도:1, 풍속:12@m/s) 기상특보 ~~> 관제탑.
  (기상청)의 (온도:2, 풍속:14@m/s) 기상특보 ~~> 관제탑.
}
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("typed frontdoor required"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_CANON_TYPED_TRANSFER_REQUIRES_SHARED_FRONTDOOR"
        );
    }

    #[test]
    fn canon_relation_eq_infix_preserves_product_path() {
        let source = r#"
관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("=:= "));
        assert!(out
            .block_editor_plan_json
            .contains("\"op\": \"relation_eq\""));
    }

    #[test]
    fn canon_accepts_receive_hooks_surface() {
        let source = r#"
관제탑:임자 = {
  기상특보를 받으면 {
    참 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    정보.온도 보여주기.
  }.

  알림을 받으면 {
    알림.이름 보여주기.
  }.

  (알림 알림.이름 == "기상특보")인 알림을 받으면 {
    알림.보낸이 보여주기.
  }.
}
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("기상특보을 받으면 {") || out.ddn.contains("기상특보를 받으면 {"));
        assert!(out
            .ddn
            .contains("(정보 정보.온도 > 40)인 기상특보를 받으면 {"));
        assert!(out.ddn.contains("알림을 받으면 {"));
        assert!(out
            .ddn
            .contains("(알림 알림.이름 == \"기상특보\")인 알림을 받으면 {"));
    }

    #[test]
    fn canon_rejects_receive_hook_outside_imja() {
        let source = r#"
기상청:움직씨 = {
  알림을 받으면 {
    참 보여주기.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_RECEIVE_OUTSIDE_IMJA");
    }

    #[test]
    fn canon_accepts_every_n_madi_hook_surface() {
        let source = r#"
(16마디)마다 {
  "tick16" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(16마디)마다 {"));
    }

    #[test]
    fn canon_rejects_every_n_madi_zero_interval() {
        let source = r#"
(0마디)마다 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_INTERVAL_INVALID");
    }

    #[test]
    fn canon_rejects_every_n_madi_unsupported_unit() {
        let source = r#"
(3 foo)마다 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED");
    }

    #[test]
    fn canon_rejects_every_n_madi_unsupported_suffix() {
        let source = r#"
(3마디)할때 {
  "bad" 보여주기.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED");
    }

    #[test]
    fn canon_normalizes_hook_start_alias_choeum_to_sijak() {
        let source = r#"
(처음)할때 {
  "boot" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(시작)할때 {"));
        assert!(!out.ddn.contains("(처음)할때 {"));
    }

    #[test]
    fn canon_normalizes_hook_tick_alias_maetic_to_maemadi() {
        let source = r#"
(매틱)마다 {
  "tick" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(매마디)마다 {"));
        assert!(!out.ddn.contains("(매틱)마다 {"));
    }

    #[test]
    fn canon_passthroughs_hook_colon_header_surface() {
        let source = r#"
(매마디)마다: {
  "tick" 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("(매마디)마다:"));
        assert!(out
            .warnings
            .iter()
            .any(|warning| warning == "W_CANON_PASSTHROUGH"));
    }

    #[test]
    fn canon_accepts_maegim_alias_and_normalizes_to_maegim() {
        let source = r#"
채비 {
  g:수 = (9.8) 조건 {
    범위: 1..20.
    간격: 0.1.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("g:수 = (9.8) 매김 {"));
        assert!(out.ddn.contains("범위: 1..20."));
        assert!(out.ddn.contains("간격: 0.1."));
    }

    #[test]
    fn canon_accepts_nested_maegim_sections_and_normalizes_canonical_sections() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      범위: 1..20.
      간격: 0.1.
    }.
    갈래 {
      가만히: 참.
      벗남다룸: "잘림".
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("g:수 = (9.8) 매김 {"));
        assert!(out.ddn.contains("가늠 {"));
        assert!(out.ddn.contains("범위: 1..20."));
        assert!(out.ddn.contains("간격: 0.1."));
        assert!(out.ddn.contains("갈래 {"));
        assert!(out.ddn.contains("가만히: 참."));
        assert!(out.ddn.contains("벗남다룸: \"잘림\"."));
        assert!(!out.ddn.contains("가늠.범위:"));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
    }

    #[test]
    fn canon_rejects_nested_maegim_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      간격: 0.1.
      분할수: 24.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT");
    }

    #[test]
    fn canon_rejects_unknown_nested_maegim_section() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    실험 {
      범위: 1..20.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_NESTED_SECTION_UNSUPPORTED");
    }

    #[test]
    fn canon_rejects_unknown_nested_maegim_field_in_ganeum() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      최소값: 1.
    }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_NESTED_FIELD_UNSUPPORTED");
    }

    #[test]
    fn canon_emits_maegim_control_plan_json() {
        let source = r#"
채비 {
  g:수 = (9.8) 조건 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .maegim_control_json
            .contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"g\""));
        assert!(out
            .maegim_control_json
            .contains("\"decl_kind\": \"butbak\""));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"theta0\""));
        assert!(out
            .maegim_control_json
            .contains("\"decl_kind\": \"gureut\""));
        assert!(out
            .maegim_control_json
            .contains("\"min_expr_canon\": \"-1.2\""));
        assert!(out
            .maegim_control_json
            .contains("\"max_expr_canon\": \"1.2\""));
        assert!(out
            .maegim_control_json
            .contains("\"split_count_expr_canon\": \"24\""));
    }

    #[test]
    fn maegim_source_plan_keeps_rust_verified_source_refs() {
        let source = "채비 {\n  초기속도x: 수 <- (8.0) 매김 { 범위: 1..30. 간격: 1. }.\n}.\n";
        let plan = maegim_control_plan_with_source(source, "lesson.ddn")
            .expect("source-aware maegim plan");
        let parsed: serde_json::Value = serde_json::from_str(&plan).expect("plan json");
        let control = &parsed["controls"][0];
        assert_eq!(control["name"], "초기속도x");
        assert!(control["source_ref"]
            .as_str()
            .expect("source_ref")
            .starts_with("lesson.ddn#bytes="));
        assert_eq!(control["source_edit"]["status"], "ready");
        let start = control["source_edit"]["byte_start"]
            .as_u64()
            .expect("byte_start") as usize;
        let end = control["source_edit"]["byte_end"]
            .as_u64()
            .expect("byte_end") as usize;
        assert_eq!(&source[start..end], "8.0");
    }

    #[test]
    fn alpha_menu_maegim_survives_full_program_canonicalization() {
        let source = include_str!("../../../solutions/seamgrim_ui_mvp/lessons/rep_phys_projectile_xy_v1/lesson.ddn");
        let output = canonicalize(source, false).expect("alpha menu canon");
        let plan: serde_json::Value = serde_json::from_str(&output.maegim_control_json).unwrap();
        assert_eq!(plan["controls"].as_array().map(Vec::len), Some(5),
            "full source controls lost; primary parse: {:?}",
            super::canonicalize_with_fallback_mode(source, false, false).err());
        assert_eq!(plan["controls"][0]["init_expr_canon"], "8.0",
            "initial value lost; primary parse: {:?}",
            super::canonicalize_with_fallback_mode(source, false, false).err());
    }

    #[test]
    fn fallback_maegim_plan_keeps_inline_literal_and_range_schema() {
        let source = "채비 {\n  기준점수: 수 := (60.0) 매김 { 범위: 30..90. 간격: 5. }.\n}.\n";
        let plan: serde_json::Value = serde_json::from_str(
            &super::build_frontdoor_fallback_maegim_json(source).unwrap(),
        ).unwrap();
        let control = &plan["controls"][0];
        assert_eq!(control["init_expr_canon"], "60.0");
        assert_eq!(control["type_name"], "수");
        assert_eq!(control["range"]["min_expr_canon"], "30");
        assert_eq!(control["range"]["max_expr_canon"], "90");
        assert_eq!(control["step_expr_canon"], "5");
    }

    #[test]
    fn fallback_maegim_lambda_program_preserves_control_and_edit_bounds() {
        let source = "채비 {\n  기준점수: 수 := (60.0) 매김 { 범위: 30..90. 간격: 5. }.\n}.\n(매마디)마다 { 원본 <- 차림[45, 60, 90]. 남긴값 <- (원본, {점수 | 점수 >= 기준점수}) 거르기. }.\n";
        let plan = maegim_control_plan_with_source(source, "lesson.ddn").unwrap();
        let plan: serde_json::Value = serde_json::from_str(&plan).unwrap();
        assert_eq!(plan["controls"].as_array().map(Vec::len), Some(1));
        assert_eq!(plan["controls"][0]["init_expr_canon"], "60.0");
        assert_eq!(plan["controls"][0]["source_edit"]["status"], "ready");
        assert_eq!(apply_maegim_source_edit(source, "lesson.ddn", "기준점수", "95")
            .expect_err("fallback range must reject out-of-range edits").code(),
            "E_MAEGIM_VALUE_OUT_OF_RANGE");
    }

    #[test]
    fn fallback_maegim_preserves_multiline_precision_and_generic_controls() {
        let source = "채비 {\n  dt: 수 := (0.000000001) 매김 {\n    범위: 0.000000001..0.1.\n    간격: 0.000000001.\n  }.\n  배수: 수 <- (2) 매김 { 범위: 1..5. 분할수: 4. }.\n  계산값: 수 <- (1 + 2) 매김 { 범위: 1..5. 간격: 1. }.\n}.\n";
        let plan: serde_json::Value = serde_json::from_str(
            &super::build_frontdoor_fallback_maegim_json(source).unwrap(),
        ).unwrap();
        assert_eq!(plan["controls"].as_array().map(Vec::len), Some(3));
        assert_eq!(plan["controls"][0]["init_expr_canon"], "0.000000001");
        assert_eq!(plan["controls"][0]["range"]["min_expr_canon"], "0.000000001");
        assert_eq!(plan["controls"][0]["step_expr_canon"], "0.000000001");
        assert_eq!(plan["controls"][1]["init_expr_canon"], "2");
        assert_eq!(plan["controls"][1]["split_count_expr_canon"], "4");
        assert!(plan["controls"][2]["init_expr_canon"].is_null());
        assert_eq!(plan["controls"][2]["name"], "계산값");
    }

    #[test]
    fn v25_definition_maegim_plan_and_source_edit_keep_definition_surface() {
        let source = "채비 {\n  초기속도x: 수 := (8.0) 매김 { 범위: 1..30. 간격: 1. }.\n}.\n";
        let plan = maegim_control_plan_with_source(source, "lesson.ddn")
            .expect("V25 source-aware maegim plan");
        let parsed: serde_json::Value = serde_json::from_str(&plan).expect("plan json");
        let control = &parsed["controls"][0];
        assert_eq!(control["name"], "초기속도x");
        assert_eq!(control["source_edit"]["status"], "ready");
        let edited = apply_maegim_source_edit(source, "lesson.ddn", "초기속도x", "12.5")
            .expect("V25 source edit");
        let edited: serde_json::Value = serde_json::from_str(&edited).expect("edit json");
        let edited_source = edited["source_text"].as_str().expect("edited source");
        assert!(edited_source.contains("초기속도x: 수 := (12.5) 매김"));
        assert!(!edited_source.contains("초기속도x: 수 <-"));
    }

    #[test]
    fn maegim_source_edit_changes_source_and_rechecks_meaning() {
        let source = "채비 {\n  초기속도x: 수 <- (8.0) 매김 { 범위: 1..30. 간격: 1. }.\n}.\n";
        let result = apply_maegim_source_edit(source, "lesson.ddn", "초기속도x", "13.5")
            .expect("source edit");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("result json");
        assert_eq!(parsed["status"], "ready");
        assert_eq!(parsed["value_canon"], "13.5");
        assert!(parsed["source_text"]
            .as_str()
            .expect("source text")
            .contains("초기속도x: 수 <- (13.5) 매김"));
        let err = apply_maegim_source_edit(source, "lesson.ddn", "초기속도x", "31")
            .expect_err("out-of-range value must fail");
        assert_eq!(err.code(), "E_MAEGIM_VALUE_OUT_OF_RANGE");
    }

    #[test]
    fn canon_emits_maegim_control_plan_json_from_seed_body_decl_block() {
        let source = r#"
매틱:움직씨 = {
  채비 {
    g:수 = (9.8) 조건 {
      범위: 1..20.
      간격: 0.1.
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .maegim_control_json
            .contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(out.maegim_control_json.contains("\"name\": \"g\""));
        assert!(out
            .maegim_control_json
            .contains("\"step_expr_canon\": \"0.1\""));
    }

    #[test]
    fn canon_emits_block_editor_plan_json_for_nested_maegim_sections() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      범위: 1..20.
      간격: 0.1.
    }.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_item_const_step\""));
    }

    #[test]
    fn canon_emits_block_editor_plan_json() {
        let source = r#"
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
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .block_editor_plan_json
            .contains("\"schema\": \"ddn.block_editor_plan.v1\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"charim_item_const_step\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"hook_start\""));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"if_else\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"for_each\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"choose_else\""));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"repeat\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"open_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"while_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"contract_guard\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_after\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_condition\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"prompt_choose\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"seed_def\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"receive_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"event_react\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"return_value\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"inspect_value\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"bogae_draw\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"break_loop\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"bogae_madang_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"exec_policy_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"jjaim_block\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"expr_stmt\""));
        assert!(out.block_editor_plan_json.contains("\"exprs\": {"));
        assert!(out.block_editor_plan_json.contains("\"kind\": \"call\""));
    }

    #[test]
    fn canon_formats_if_as_manyak_iramyeon() {
        let source = r#"
(1 < 2) 일때 {
  살림.x <- 1.
} 아니면 {
  살림.x <- 2.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("만약 1 < 2 이라면 {"));
        assert!(!out.ddn.contains(" 일때 {"));
    }

    #[test]
    fn canon_accepts_manyak_if_with_dot_newline_else() {
        let source = r#"
만약 1 < 2 이라면 {
  살림.x <- 1.
}.
아니면 {
  살림.x <- 2.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("만약 1 < 2 이라면 {"));
        assert!(out.ddn.contains("} 아니면 {"));
    }

    #[test]
    fn canon_accepts_choose_plain_condition_without_fallback() {
        let source = r#"
고르기:
1 < 2: {
  살림.x <- 1.
}
아니면: {
  살림.x <- 2.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("고르기:"));
        assert!(out.ddn.contains("1 < 2: {"));
        assert!(out.ddn.contains("아니면: {"));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"choose_else\""));
        assert!(!out
            .warnings
            .iter()
            .any(|warning| warning.contains("W_CANON_FALLBACK")));
    }

    #[test]
    fn canon_accepts_choose_plain_condition_in_case_with_geubakke() {
        let source = r#"
고르기:
1 < 2 인 경우 {
  살림.x <- 1.
}
그밖의 경우 {
  살림.x <- 2.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("고르기:"));
        assert!(out.ddn.contains("1 < 2: {"));
        assert!(out.ddn.contains("아니면: {"));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"choose_else\""));
        assert!(!out
            .warnings
            .iter()
            .any(|warning| warning.contains("W_CANON_FALLBACK")));
    }

    #[test]
    fn canon_accepts_choose_plain_condition_in_case_exhaustive() {
        let source = r#"
고르기:
1 < 2 인 경우 {
  살림.x <- 1.
}
모든 경우 다룸.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("고르기:"));
        assert!(out.ddn.contains("1 < 2: {"));
        assert!(out.ddn.contains("모든 경우 다룸."));
        assert!(!out.ddn.contains("아니면: {"));
        assert!(out
            .block_editor_plan_json
            .contains("\"kind\": \"choose_exhaustive\""));
        assert!(out
            .block_editor_plan_json
            .contains("\"exhaustive\": \"true\""));
        assert!(!out
            .warnings
            .iter()
            .any(|warning| warning.contains("W_CANON_FALLBACK")));
    }

    #[test]
    fn canon_rejects_maegim_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
    분할수: 24.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT");
    }

    #[test]
    fn canon_accepts_decl_value_prefix_call_with_parenthesized_args() {
        let source = r#"
채비 {
  목록:차림 <- (1, 2) 차림.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("목록:차림 <- (1, 2) 차림."));
    }

    #[test]
    fn canon_accepts_block_header_colon_with_foreach_and_prefix_call() {
        let source = r#"
채비: {
  목록:차림 <- (1, 2) 차림.
}.

매마디:움직씨 = {
  반복: {
    (x) 목록에 대해: {
      x 보여주기.
    }.
    멈추기.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("목록:차림 <- (1, 2) 차림."));
        assert!(out.ddn.contains("되풀이 {"));
        assert!(out.ddn.contains("(x) 목록에 대해 {"));
        assert!(out
            .warnings
            .iter()
            .any(|w| w.contains("W_BLOCK_HEADER_COLON_DEPRECATED")));
    }

    #[test]
    fn canon_routes_natural_iteration_and_current_item_through_shared_lang() {
        let source = r#"
(값:수) 기록:움직씨 = { 값 보여주기. }
테스트:움직씨 = {
  채비 { 목록:(수)차림 <- (1, 2) 차림. }.
  목록마다 { () 기록하기. }.
}
"#;
        let out = canonicalize(source, false).expect("shared natural iteration canon");
        assert!(out.ddn.contains("(목록:수) 목록에 대해 {"));
        assert!(out.ddn.contains("목록:값 기록기."));
        assert!(!out.ddn.contains("목록마다"));
        assert!(!out
            .warnings
            .iter()
            .any(|warning| warning.contains("W_CANON_FALLBACK")));
    }

    #[test]
    fn canon_natural_iteration_preserves_shared_fail_closed_diagnostics() {
        let source = r#"
(왼쪽:수, 오른쪽:수) 기록:움직씨 = { 없음. }
테스트:움직씨 = {
  채비 { 목록:(수)차림 <- (1) 차림. }.
  목록마다 { () 기록하기. }.
}
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("ambiguous current item must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CURRENT_ITEM_AMBIGUOUS");
    }

    #[test]
    fn canon_rejects_decl_block_in_repeat_body() {
        let source = r#"
테스트:움직씨 = {
  반복 {
    채비 { 값:수 <- 0. }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CHAEBI_IN_LOOP");
    }

    #[test]
    fn canon_rejects_decl_block_in_hook_body() {
        let source = r#"
테스트:움직씨 = {
  (처음)할때 {
    채비 { 값:수 <- 0. }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CHAEBI_IN_LOOP");
    }

    #[test]
    fn canon_rejects_decl_block_in_foreach_body() {
        let source = r#"
테스트:움직씨 = {
  채비 {
    값목록:차림 <- [1, 2, 3].
  }.
  (x) 값목록에 대해 {
    채비 { 값:수 <- x. }.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CHAEBI_IN_LOOP");
    }

    #[test]
    fn canon_warns_top_level_chaebi_reassign() {
        let source = r#"
매마디:움직씨 = {
  채비 {
    값:수 <- 0.
  }.
  값 <- 1.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .warnings
            .iter()
            .any(|w| w.contains("W_CHAEBI_REDUNDANT_TOP_REASSIGN")));
    }

    #[test]
    fn canon_skips_top_level_chaebi_derived_reassign_warning() {
        let source = r#"
매마디:움직씨 = {
  채비 {
    점수:수 <- 72.
    통과함:참거짓 <- 거짓.
  }.
  통과함 <- 점수 >= 70.
  통과함 보여주기.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(!out
            .warnings
            .iter()
            .any(|w| w.contains("W_CHAEBI_REDUNDANT_TOP_REASSIGN")));
    }

    #[test]
    fn canon_normalizes_repeat_and_return_surface_to_canonical_keywords() {
        let source = r#"
테스트:셈씨 = {
  반복: {
    1 반환.
  }.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("되풀이 {"));
        assert!(out.ddn.contains("1 되돌림."));
        assert!(!out.ddn.contains("반복 {"));
        assert!(!out.ddn.contains("1 반환."));
    }

    #[test]
    fn canon_accepts_unary_plus_literals() {
        let source = r#"
채비 {
  x:수 <- +5.
  y:수 <- (+5).
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("x:수 <- 5."));
        assert!(out.ddn.contains("y:수 <- 5."));
    }

    #[test]
    fn canon_normalizes_audit_surface_to_canonical_keyword() {
        let source = r#"
테스트:셈씨 = {
  값 감사.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("값 톺아보기."));
        assert!(!out.ddn.contains("값 감사."));
    }

    #[test]
    fn canon_normalizes_legacy_bogae_jangmyeon_block() {
        let source = r#"
보개장면 {
  #자막("legacy").
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("보개마당 {"));
        assert!(!out.ddn.contains("보개장면 {"));
    }

    #[test]
    fn canon_frontdoor_accepts_file_leading_setting_block() {
        let source = r#"
설정 {
  문서 {
    이름: "프론트도어".
  }.
}.

채비 {
  x:수 <- 1.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out.ddn.contains("채비 {"));
        assert!(out.ddn.contains("x:수 <- 1."));
        assert!(!out.ddn.contains("설정 {"));
    }

    #[test]
    fn exec_policy_surface_detector_distinguishes_presence() {
        let no_policy = r#"
채비 {
  너머값:수 <- 1.
}.
"#;
        assert!(!has_exec_policy_surface(no_policy));

        let with_policy = r#"
너머 {
  실행모드: 엄밀.
  효과정책: 격리.
}.
"#;
        assert!(has_exec_policy_surface(with_policy));
    }

    #[test]
    fn canon_exec_policy_map_emits_strict_default_contract_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let out = canonicalize(source, false).expect("canonicalize");
        assert!(out
            .exec_policy_map_json
            .contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert!(out.exec_policy_map_json.contains("\"block_count\": 0"));
        assert!(out
            .exec_policy_map_json
            .contains("\"exec_mode_effective\": \"엄밀\""));
        assert!(out
            .exec_policy_map_json
            .contains("\"effect_policy_effective\": \"격리\""));
    }

    #[test]
    fn canon_rejects_legacy_hash_header_on_frontdoor() {
        let source = r#"
#이름: 금지
채비 {
  x:수 <- 1.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("must reject"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN");
    }

    #[test]
    fn canon_rejects_legacy_boim_surface_on_frontdoor() {
        let source = r#"
(매마디)마다 {
  보임 {
    x: 1.
  }.
}.
"#;
        let err = match canonicalize(source, false) {
            Ok(_) => panic!("legacy boim must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_BOIM_LEGACY_REMOVED");
    }
