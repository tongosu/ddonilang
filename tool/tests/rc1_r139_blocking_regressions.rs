use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;

fn run(source: &str) -> Result<serde_json::Value, String> {
    run_summary_from_supported_source_profile(source, "rc1-blocking.ddn", Some(1), Some("v1-core-v25"))
}

#[test]
fn rd315_public_observation_executes_without_builtin_registration() {
    run("t := 0. 값 := 1. (x축: t, 값: 값) 내보임.").expect("관찰 선언 실행");
}

#[test]
fn grid_score_intermediate_overflow_is_an_explicit_atomic_failure() {
    let source = r#"점수 <- (__종류: "std_grid_game_score", 점수: 0, 줄수: ("9223372036854775807") 큰바른수). 결과 <- (점수,4) 격자게임점수.더하기. 결과 보여주기."#;
    let error = run(source).expect_err("넘침을 점수160 성공으로 발행하면 안 됨");
    assert!(error.contains("격자게임점수 중간 곱셈"), "{error}");
    assert!(error.contains("E_RUNTIME_EXECUTION"), "{error}");
}

#[test]
fn grid_score_normal_result_is_unchanged() {
    let actual = run("점수 <- () 격자게임점수.초기화. 결과 <- (점수,4) 격자게임점수.더하기. 결과.점수 보여주기.").expect("정상 점수");
    assert!(actual.to_string().contains("800"), "{actual}");
}
