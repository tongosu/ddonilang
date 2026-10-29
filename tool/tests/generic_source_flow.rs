use ddonirang_tool::ddn_runtime::{DdnProgram, DdnRunner};
use ddonirang_core::{Fixed64, InputSnapshot, InputSource};
use ddonirang_core::platform::NuriWorld;
use std::collections::HashMap;

const ID: &str = "(값씨:씨자리) 두고 (값:값씨~을) 그대로:값씨:셈씨 := { 그대로 <- 값. }.";

fn run(raw: &str) -> Result<Vec<String>, String> {
    let program = DdnProgram::from_v25_source(raw,"제네릭.ddn")?;
    let input = InputSnapshot { tick_id:0, dt:Fixed64::from_i64(1), keys_pressed:0,
        last_key_name:String::new(), pointer_x_i32:0, pointer_y_i32:0,
        ai_injections:Vec::new(), net_events:Vec::new(), frame_source:InputSource::Person, rng_seed:0 };
    let output = DdnRunner::new(program,"매마디").run_update(&NuriWorld::new(),&input,&HashMap::new())?;
    Ok(output.display_outputs.into_iter().map(|row|row.text).collect())
}

#[test]
fn generic_source_distinct_calls_execute_with_distinct_types() {
    let raw=format!("{ID}\n((값=3) 그대로) 보여주기. ((값=\"글\") 그대로) 보여주기.");
    assert_eq!(run(&raw).unwrap(), vec!["3","글"]);
}

#[test]
fn generic_source_conflict_reports_both_input_locations() {
    let raw="(T:씨자리) 두고 (앞:T~을, 뒤:T~로) 첫째:T:셈씨 := { 첫째 <- 앞. }.\n결과 <- (앞=3, 뒤=\"글\") 첫째.";
    let error=run(raw).unwrap_err();
    assert!(error.contains("E_GENERIC_UNIFY_CONFLICT:"),"{error}");
    for fragment in ["3","\"글\""] {
        let start=raw.find(fragment).unwrap();
        assert!(error.contains(&format!("{}..{}",start,start+fragment.len())),"{error}");
    }
}

#[test]
fn generic_source_body_cannot_use_coincident_actual_types_as_proof() {
    let raw="(T:씨자리, U:씨자리) 두고 (앞:T~을, 뒤:U~로) 첫째:T:셈씨 := { 첫째 <- 뒤. }.\n결과 <- (앞=3, 뒤=4) 첫째.";
    assert!(run(raw).unwrap_err().contains("E_GENERIC_RESULT_TYPE:"));
}

#[test]
fn generic_source_unresolved_and_unsupported_body_are_rejected() {
    for raw in [ID.replace("값씨:씨자리","값씨:씨자리, 미결:씨자리")+" 결과 <- (값=3) 그대로.",
        ID.replace("그대로 <- 값.","그대로 <- 3.")+" 결과 <- (값=3) 그대로."] {
        assert!(run(&raw).is_err());
    }
}

#[test]
fn generic_source_return_context_cannot_choose_input_type() {
    let raw=format!("{ID}\n계산:나눔수:셈씨 := {{ 계산 <- (값=3) 그대로. }}. (() 계산) 보여주기.");
    assert!(run(&raw).is_err(), "반환 문맥으로 씨자리를 나눔수로 바꾸면 안 됩니다");
}

#[test]
fn generic_source_missing_required_input_is_rejected() {
    let raw=format!("{ID}\n(() 그대로) 보여주기.");
    assert!(run(&raw).is_err());
}

#[test]
fn generic_source_common_diagnostics_preserve_code_and_both_utf8_spans() {
    let raw="// 한글 😀\r\n(T:씨자리) 두고 (앞:T~을, 뒤:T~로) 첫째:T:셈씨 := { 첫째 <- 앞. }.\r\n결과 <- (앞=3, 뒤=\"글\") 첫째.";
    let snapshot=ddonirang_tool::source_analysis::analyze_source(ddonirang_tool::source_analysis::SourceAnalysisRequest {
        source_text:raw.into(),path:"진단.ddn".into(),analysis_generation:7,source_profile_identity:"v1-core-v25".into(),
    });
    assert_eq!(snapshot.outcome,"REJECTED");
    assert_eq!(snapshot.diagnostics.len(),2);
    for (diagnostic,fragment) in snapshot.diagnostics.iter().zip(["3","\"글\""]) {
        assert_eq!(diagnostic.code,"E_GENERIC_UNIFY_CONFLICT");
        assert_eq!(&raw[diagnostic.span.start..diagnostic.span.end],fragment);
    }
    let error=ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile(
        raw,"진단.ddn",Some(1),Some("v1-core-v25")).unwrap_err();
    let diagnostic:serde_json::Value=serde_json::from_str(&error).unwrap();
    assert_eq!(diagnostic["code"],"E_GENERIC_UNIFY_CONFLICT");
    let range=|value:&serde_json::Value|value["start"].as_u64().unwrap() as usize..value["end"].as_u64().unwrap() as usize;
    assert_eq!(&raw[range(&diagnostic["source_span"])],"\"글\"");
    assert_eq!(&raw[range(&diagnostic["cause"]["source_span"])],"3");
}
