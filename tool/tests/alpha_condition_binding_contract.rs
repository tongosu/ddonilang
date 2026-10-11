use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;

fn run(source: &str) -> Result<serde_json::Value, String> {
    run_summary_from_supported_source_profile(source, "조건과결속.ddn", Some(1), Some("v1-core-v25"))
}

fn placements(body: &str) -> [String; 2] {
    [body.to_owned(), format!("(매마디)마다 {{ {body} }}.")]
}

#[test]
fn non_boolean_if_conditions_are_rejected_without_a_success_summary() {
    for condition in ["0", "1", "없음", "\"글\""] {
        for source in placements(&format!(
            "{{ {condition} }}이면 {{ 100 보여주기. }} 아니면 {{ 200 보여주기. }}."
        )) {
            let error = run(&source).expect_err("참거짓이 아닌 조건은 실행할 수 없습니다");
            assert!(error.contains("참거짓"), "{source}: {error}");
            assert!(!error.contains("output_log_texts"), "{source}: {error}");
        }
    }
}

#[test]
fn non_boolean_while_conditions_are_rejected_even_when_falsy() {
    for condition in ["0", "없음"] {
        for source in placements(&format!("{{ {condition} }}인 동안 {{ 100 보여주기. }}.")) {
            let error = run(&source).expect_err("거짓처럼 취급하던 값도 참거짓 형식이어야 합니다");
            assert!(error.contains("참거짓"), "{source}: {error}");
        }
    }
}

#[test]
fn boolean_conditions_keep_branch_and_loop_behavior() {
    // Owner D6 (2026-10-01): migrate only spelling, retain the same assertions.
    let body = "{ 참 }이면 { 100 보여주기. } 아니면 { 200 보여주기. }. \
                { 거짓 }이면 { 300 보여주기. } 아니면 { 400 보여주기. }. \
                값:수 := 0. { 값 < 2 }인 동안 { 값 <- 값 + 1. }. 값 보여주기.";
    for source in placements(body) {
        let summary = run(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(summary["output_log_texts"], serde_json::json!(["100", "400", "2"]));
    }
}

#[test]
fn explicit_declared_type_rejects_different_type_rebinding() {
    for body in [
        "값:수 := 1. 값 <- \"가\". 값 보여주기.",
        "값:글 := \"가\". 값 <- 1. 값 보여주기.",
    ] {
        for source in placements(body) {
            let error = run(&source).expect_err("명시한 형식과 다른 값으로 재결속할 수 없습니다");
            // WO02 a03c337a: shared CLI/WASM validation rejects known types before execution.
            assert!(error.contains("E_CHECK_TYPE_MISMATCH"), "{source}: {error}");
            assert!(!error.contains("output_log_texts"), "{source}: {error}");
        }
    }
}

#[test]
fn explicit_declared_type_allows_same_type_rebinding() {
    for source in placements("값:수 := 1. 값 <- 2. 글값:글 := \"가\". 글값 <- \"나\". 값 보여주기. 글값 보여주기.") {
        let summary = run(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(summary["output_log_texts"], serde_json::json!(["2", "나"]));
    }
}

#[test]
fn owner_approved_wo02_inferred_type_rebinding_is_rejected() {
    for source in placements("값 := 1. 값 <- \"가\". 값 보여주기.") {
        // Owner D1 approval 2026-09-30 supersedes the R24 untyped behavior.
        let error = run(&source).expect_err("추론한 씨도 다른 씨로 재결속할 수 없습니다");
        assert!(error.contains("E_CHECK_TYPE_MISMATCH"), "{source}: {error}");
        assert!(!error.contains("output_log_texts"), "{source}: {error}");
    }
}

#[test]
fn explicit_collection_element_type_is_checked_when_replacing_an_element() {
    for source in placements("판:(수)차림 := 차림[0]. \"가\" -> 판[0]. 판 보여주기.") {
        let error = run(&source).expect_err("원소 교체도 명시한 차림 원소 형식을 지켜야 합니다");
        assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"), "{source}: {error}");
    }
    for source in placements("판:(수)차림 := 차림[0]. 2 -> 판[0]. 판[0] 보여주기.") {
        let summary = run(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(summary["output_log_texts"], serde_json::json!(["2"]));
    }
}

#[test]
fn foreach_binder_shadows_then_restores_outer_explicit_type_and_value() {
    let prefix = "항목:수 := 7. 대상 := 차림[\"가\", \"나\"]. \
                  (항목) 대상에 대해 { 항목 보여주기. 항목 <- \"다\". 항목 보여주기. }. 항목 보여주기.";
    for source in placements(&format!("{prefix} 항목 <- 8. 항목 보여주기.")) {
        let summary = run(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(summary["output_log_texts"], serde_json::json!(["가", "다", "나", "다", "7", "8"]));
    }
    for source in placements(&format!("{prefix} 항목 <- \"바깥 형식 위반\".")) {
        let error = run(&source).expect_err("순회 뒤에는 바깥 이름의 명시 형식이 복원되어야 합니다");
        assert!(error.contains("E_CHECK_TYPE_MISMATCH"), "{source}: {error}");
    }
}

#[test]
fn callable_local_explicit_type_is_enforced_without_leaking_to_the_caller() {
    let invalid = "계산:수:셈씨 := { 중간:수 := 1. 중간 <- \"가\". 계산 <- 2. }. () 계산 보여주기.";
    let error = run(invalid).expect_err("호출 본문의 명시 형식도 재결속에 적용되어야 합니다");
    assert!(error.contains("E_CHECK_TYPE_MISMATCH"), "{error}");

    let valid = "계산:수:셈씨 := { 중간:수 := 1. 중간 <- 2. 계산 <- 중간. }. \
                 중간:글 := \"가\". () 계산 보여주기. 중간 <- \"나\". 중간 보여주기.";
    let summary = run(valid).unwrap_or_else(|error| panic!("{valid}: {error}"));
    assert_eq!(summary["output_log_texts"], serde_json::json!(["2", "나"]));
}

#[test]
fn callable_parameter_explicit_type_is_enforced_on_rebinding() {
    let invalid = "(값:수~을~를) 더하:수:셈씨 := { 값 <- \"가\". 더하 <- 2. }. 1을 더하기 보여주기.";
    let error = run(invalid).expect_err("받는자리의 명시 형식도 재결속에 적용되어야 합니다");
    assert!(error.contains("E_CHECK_TYPE_MISMATCH"), "{error}");

    let valid = "(값:수~을~를) 더하:수:셈씨 := { 값 <- 값 + 1. 더하 <- 값. }. 1을 더하기 보여주기.";
    let summary = run(valid).unwrap_or_else(|error| panic!("{valid}: {error}"));
    assert_eq!(summary["output_log_texts"], serde_json::json!(["2"]));
}
