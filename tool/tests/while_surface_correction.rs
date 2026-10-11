use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;

fn run(source: &str) -> Result<serde_json::Value, String> {
    run_summary_from_supported_source_profile(source, "동안.ddn", Some(1), Some("v1-core-v25"))
}

#[test]
fn aborted_root_loops_terminate_without_publishing_a_summary() {
    let abort = "값 <- 99. { 거짓 }인것 바탕으로(물림) 아니면 { 값 <- 100. }.";
    for body in [
        format!("되풀이 {{ {abort} }}."),
        format!("{{ 참 }}인 동안 {{ {abort} }}."),
        format!("되풀이 {{ {{ 참 }}인 동안 {{ {abort} }}. }}."),
        format!("{{ 참 }}인 동안 {{ 되풀이 {{ {abort} }}. }}."),
        "결과 <- { 되풀이 { { 거짓 }인것 바탕으로(물림) 아니면 { }. }. 1. }한것.".to_string(),
        "결과 <- { { 참 }인 동안 { { 거짓 }인것 바탕으로(물림) 아니면 { }. }. 1. }한것.".to_string(),
    ] {
        let source = format!("값 <- 7. \"게시 금지\" 보여주기. {body} \"반복 뒤 게시 금지\" 보여주기.");
        let error = run(&source).expect_err("중단된 프로그램은 성공 요약을 게시하면 안 됩니다");
        assert!(error.contains("E_ROOT_PROGRAM_ABORTED"), "{source}: {error}");
        for field in ["output_log_texts", "state_hash", "public_observation_trace_hash", "work_cost_receipt"] {
            assert!(!error.contains(field), "중단 오류에 성공 결과가 포함됨: {error}");
        }
    }
}

fn program(body: &str) -> String {
    format!("매마디:움직씨 := {{\n{body}\n}}.")
}

#[test]
fn ordinary_while_executes_zero_multiple_and_nested_iterations() {
    let cases = [
        ("값:수 := 7.\n{ 거짓 }인 동안 { 값 <- 99. }.\n값 보여주기.", "7"),
        ("값:수 := 0.\n{ 값 < 2 }인 동안 { 값 <- 값 + 1. }.\n{ 값 < 4 }인 동안 { 값 <- 값 + 1. }.\n값 보여주기.", "4"),
        ("바깥:수 := 0.\n안쪽:수 := 0.\n{ 바깥 < 2 }인 동안 { 안쪽 <- 0. { 안쪽 < 2 }인 동안 { 안쪽 <- 안쪽 + 1. }. 바깥 <- 바깥 + 1. }.\n(바깥 + 안쪽) 보여주기.", "4"),
    ];
    for (body, expected) in cases {
        let summary = run(&program(body)).unwrap();
        assert_eq!(summary["output_log_texts"], serde_json::json!([expected]));
    }
}

#[test]
fn legacy_colon_and_non_boolean_while_fail_without_a_success_summary() {
    for body in [
        "{ 참 }인것 동안 { }.",
        "{ 참 }인것 동안: { }.",
        "{ 참 }인 동안: { }.",
        "{ 없는값 }인 동안 { }.",
    ] {
        assert!(run(&program(body)).is_err(), "{body}");
    }
}

#[test]
fn shared_canon_emits_only_the_current_while_surface() {
    let source = program("값:수 := 0.\n{ 값 < 2 }인 동안 { 값 <- 값 + 1. }.");
    let first = ddonirang_tool::canon::canonicalize(&source, false).unwrap();
    assert!(first.ddn.contains("인 동안"));
    assert!(!first.ddn.contains("인것 동안"));
    let second = ddonirang_tool::canon::canonicalize(&first.ddn, false).unwrap();
    assert_eq!(first.ddn, second.ddn);

    for legacy in ["{ 참 }인것 동안 { }.", "{ 참 }인것 동안: { }."] {
        if let Ok(output) = ddonirang_tool::canon::canonicalize(&program(legacy), false) {
            panic!("{legacy} accepted as {}", output.ddn);
        }
    }
}
