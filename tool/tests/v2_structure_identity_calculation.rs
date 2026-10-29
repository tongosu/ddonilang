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
fn initial_identity_executes_actual_seven() {
    let source=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let source=format!("{source}\n((값=7, 근거=항등구조.법칙) 법칙필수계산) 보여주기.");
    assert_eq!(run(&source).unwrap(),vec!["7"]);
}
#[test]
fn constant_seven_is_not_an_identity() {
    let source=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let source=format!("{}\n((값=7, 근거=항등구조.법칙) 법칙필수계산) 보여주기.",source.replace("그대로 <- 값.","그대로 <- 7."));
    assert!(run(&source).is_err());
}
#[test]
fn s4_frontdoor_static_roles_do_not_whitelist_ordinary_names() {
    use ddonirang_tool::source_frontdoor_workflow::validate_supported_source_for_execution as check;
    let file=std::path::Path::new("s4.ddn");
    let base=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let raw=format!("{base}\n((값=7, 근거=항등구조.법칙) 법칙필수계산) 보여주기.");
    check(&raw,file,"v1-core-v25").unwrap();
    for bad in ["바른수 보여주기.".to_string(),
        "(바탕씨=바른수) 없는도우미 보여주기.".to_string(),
        format!("항등법칙요구짜임 := 1.\n{raw}"),
        raw.replace("바탕씨=바른수","바탕씨=나눔수"),
        raw.replace("근거=항등구조.법칙","근거=참")] {
        assert!(check(&bad,file,"v1-core-v25").is_err(),"{bad}");
    }
}

#[test]
fn s4_actual_module_entry_uses_checked_identity() {
    let base=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let module=format!("{base}\n실행:움직씨 := {{ ((값=7, 근거=항등구조.법칙) 법칙필수계산) 보여주기. }}.");
    let input=serde_json::json!({"schema":"ddn.virtual_project_source.v1","root_path":"root.ddn","entry_path":"app.ddn","files":[
        {"path":"root.ddn","role":"root","utf8_text":"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음."},
        {"path":"app.ddn","role":"ddn_source","utf8_text":module}]}).to_string();
    let result=ddonirang_tool::local_project::build_project_run_summary_with_supported_profile_json(&input,"",false,false).unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&result).unwrap()["runtime_output_log_texts"],serde_json::json!(["7"]));
}

fn module_request(body:&str,start:&str)->String {
    serde_json::json!({"schema":"ddn.virtual_project_source.v1","root_path":"root.ddn","entry_path":"app.ddn","files":[
        {"path":"root.ddn","role":"root","utf8_text":format!("(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app.ddn\", 원문=\"app.ddn\")], 시작=(모듈=앱, 정의={start})) 얽음.")},
        {"path":"app.ddn","role":"ddn_source","utf8_text":body}]}).to_string()
}
#[test]
fn s4_module_keeps_selected_entry_and_rejects_scope_bypass() {
    use ddonirang_tool::local_project::build_project_run_summary_with_supported_profile_json as check;
    let base=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let call="((값=7, 근거=항등구조.법칙) 법칙필수계산) 보여주기.";
    let raw=format!("{base}\n실행:움직씨 := {{ {call} }}.");
    let other=format!("{raw}\n다른실행:움직씨 := {{ {} }}.",call.replace("값=7","값=23"));
    let result=check(&module_request(&other,"다른실행"),"",false,false).unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&result).unwrap()["runtime_output_log_texts"],serde_json::json!(["23"]));
    for bad in [raw.replace("그대로 <- 값.","그대로 <- 7."),
        raw.replace("근거=항등구조.법칙","근거=참"),
        raw.replace("실행:움직씨 := {","실행:움직씨 := { 항등구조 := 7."),
        format!("{raw}\n{call}")] {
        assert!(check(&module_request(&bad,"실행"),"",false,false).is_err());
    }
    assert!(check(&module_request(&raw,"실행"),"다른실행",false,false).unwrap_err().code.contains("START_OVERRIDE"));
}
