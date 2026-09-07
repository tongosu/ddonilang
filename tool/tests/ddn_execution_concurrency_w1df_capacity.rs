use ddonirang_core::platform::NuriWorld;
use ddonirang_core::{Fixed64, InputSnapshot, InputSource};
use ddonirang_lang::runtime::Value as RuntimeValue;
use ddonirang_tool::ddn_runtime::{DdnProgram, DdnRunner};
use std::collections::HashMap;

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
        frame_source: InputSource::Person,
        rng_seed: 0,
    }
}

#[test]
fn successor_async_capacity_two_multiple_waiters_resume_fifo_without_drop() {
    let script = r#"
채비 { 순서:수 <- 0. }.
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
(값:수) 셋알림:알림씨 = { 없음. }
(값:수) 넷알림:알림씨 = { 없음. }
(값:수) 다섯알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 {
    순서 <- 순서 * 10 + 1.
    ((값=0) 둘알림) ~> 제.
    ((값=0) 셋알림) ~> 제.
    ((값=0) 넷알림) ~> 제.
    ((값=0) 다섯알림) ~> 제.
    순서 <- 순서 * 10 + 2.
  }.
  (받은알림:둘알림)를 받으면 { 순서 <- 순서 * 10 + 3. }.
  (받은알림:셋알림)를 받으면 { 순서 <- 순서 * 10 + 4. }.
  (받은알림:넷알림)를 받으면 { 순서 <- 순서 * 10 + 5. }.
  (받은알림:다섯알림)를 받으면 { 순서 <- 순서 * 10 + 6. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
    let program = DdnProgram::from_source(script, "successor_async_capacity_two.ddn")
        .expect("parse");
    let mut defaults = HashMap::new();
    defaults.insert(
        "순서".to_string(),
        RuntimeValue::Fixed64(Fixed64::from_i64(0)),
    );
    let output = DdnRunner::new(program, "매틱")
        .with_async_queue_capacity(2)
        .run_update(&NuriWorld::new(), &empty_input(), &defaults)
        .expect("capacity two WAIT resumes");
    assert_eq!(
        output.resources.get("순서"),
        Some(&RuntimeValue::Fixed64(Fixed64::from_i64(123456)))
    );
}
