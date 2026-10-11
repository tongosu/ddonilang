use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;

fn run(source: &str) -> Result<serde_json::Value, String> {
    run_summary_from_supported_source_profile(source, "일반프로그램.ddn", Some(1), Some("v1-core-v25"))
}

#[test]
fn natural_call_and_short_foreach_execute_identically_at_root_and_in_body() {
    let declaration = "(사용량:수~을~를, 기본요금:수~에, 단가:수~로~으로) 요금계산:수:셈씨 := { 요금계산 <- 기본요금 + 사용량 * 단가. }.";
    for (prefix, body, expected) in [
        (declaration, "결과 := 120을 1000에 15로 요금계산하기. 결과 보여주기.", serde_json::json!(["2800"])),
        ("", "입력들:(수)차림 := 차림[1, 2, 3]. 입력들마다 { 입력들 보여주기. }.", serde_json::json!(["1", "2", "3"])),
    ] {
        for source in [format!("{prefix}\n{body}"), format!("{prefix}\n(매마디)마다 {{ {body} }}.")] {
            let summary = run(&source).unwrap_or_else(|error| panic!("{source}\n{error}"));
            assert_eq!(summary["output_log_texts"], expected, "{source}");
        }
    }
}

#[test]
fn root_selection_reaches_the_same_range_check_as_body_selection() {
    let body = "값들 := 차림[1, 2]. 값들[5] 보여주기.";
    for source in [body.to_string(), format!("(매마디)마다 {{ {body} }}.")] {
        let error = run(&source).unwrap_err();
        assert!(error.contains("CHARIM_INDEX_OUT_OF_RANGE"), "{source}: {error}");
    }
}
