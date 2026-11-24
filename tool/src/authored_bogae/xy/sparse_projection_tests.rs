//! Deliberate status mutations of a real execution trace. These tests exercise
//! the Rust projector, not public sparse-source or branch production support.
use super::*;
use ddonirang_lang::runtime::PlaneElement;
use ddonirang_lang::stdlib::xy_registration_r1::{checked_descriptor, XyMode};

fn counts(scene: &ddonirang_lang::runtime::PlaneScene) -> (usize, usize) {
    (scene.elements.iter().filter(|e| matches!(e, PlaneElement::Circle { .. })).count(),
     scene.elements.iter().filter(|e| matches!(e, PlaneElement::Polygon { .. })).count())
}

#[test]
fn supplied_sparse_cells_break_lines_without_changing_sample_order_or_exact_values() {
    let source = "설정 { 마디수: 5. }. 실행:움직씨 := { }. \
        (시작)할때 { 경과 <- 0. 위치 <- 0. }. \
        (매마디)마다 { 경과 <- 경과 + 1. 위치 <- 위치 + 2. }. \
        (경과:경과 * 1@s,위치:위치 * 1@m) 내보임.";
    let result = crate::runtime_surface::run_summary_from_supported_source_profile(
        source, "xy.ddn", Some(5), Some("v1-core-v25")).unwrap();
    let trace = result["public_observation_trace"].clone();
    assert_eq!(trace["samples"].as_array().unwrap().len(), 5);
    let run = trace["run_id"].as_str().unwrap();
    let selection = super::tests::axes(source, "관찰.경과", "관찰.위치").unwrap()
        .into_typed(&checked_descriptor(XyMode::Polyline).unwrap()).unwrap();
    assert_eq!(counts(&project_joint(&selection, &trace, run).unwrap().0), (5, 4));

    for state in ["gap", "missing", "pruned"] {
        for axis in 0..2 {
            let mut sparse = trace.clone();
            let cell = sparse["samples"][2]["cells"][axis].as_object_mut().unwrap();
            cell.remove("value_canon");
            cell.remove("value_text");
            cell.insert("status".into(), serde_json::json!(state));
            if state == "missing" {
                cell.insert("reason".into(), serde_json::json!("#원천없음"));
            }
            let before = sparse.clone();
            let (scene, candidate) = project_joint(&selection, &sparse, run).unwrap();
            // Two observed pairs on either side: no bridging segment across
            // the supplied nonpresent sample, and no replacement zero point.
            assert_eq!(counts(&scene), (4, 2), "{state} axis {axis}");
            assert_eq!(candidate.samples.len(), 5);
            assert_eq!(candidate.samples[2].cells[axis]["status"], state);
            for (index, (sample, supplied)) in candidate.samples.iter().zip(sparse["samples"].as_array().unwrap()).enumerate() {
                for axis in 0..2 {
                    for field in ["status", "reason", "value_canon", "value_text", "dataset_target"] {
                        assert_eq!(sample.cells[axis].get(field), supplied["cells"][axis].get(field));
                    }
                }
                assert_eq!(sample.display_event_index, index as u64);
                assert_eq!(sample.committed_madi, supplied["committed_madi"].as_u64().unwrap());
                assert_eq!(sample.state_hash, supplied["state_hash"].as_str().unwrap());
            }
            assert_eq!(sparse, before);
        }
    }
}

#[test]
fn typed_axis_rejects_self_consistent_trace_with_the_wrong_unit_dimension() {
    let source = super::tests::SOURCE;
    let trace = super::tests::trace();
    let run = trace["run_id"].as_str().unwrap();
    let axes = super::tests::axes(source, "관찰.경과", "관찰.위치").unwrap();
    let mut wrong = trace.clone();
    wrong["columns"][0]["unit"] = serde_json::json!("m");
    let cell = &mut wrong["samples"][0]["cells"][0];
    cell["unit"] = serde_json::json!("m");
    cell["value_canon"] = serde_json::json!("1@m");
    cell["value_text"] = serde_json::json!("1@m");
    // This is internally consistent JSON, but the real source's checked X
    // axis is time. A label/unit swap must not override that typed contract.
    assert!(crate::deterministic_debug::build_public_observation_table_projection(&wrong, run).is_ok());
    let error = produce_joint_candidate(&axes, &wrong, run, None).err().expect("wrong dimension must be rejected");
    assert!(error.contains("단위 차원이 원문 타입과 다릅니다"), "{error}");

    let mut wrong_canonical = trace.clone();
    wrong_canonical["samples"][0]["cells"][0]["value_canon"] = serde_json::json!("1@m");
    let error = produce_joint_candidate(&axes, &wrong_canonical, run, None).err().expect("canonical unit must match its column");
    assert!(error.contains("정확값과 공급 열의 단위가 다릅니다"), "{error}");

    let mut wrong_type = trace.clone();
    wrong_type["columns"][0]["dtype"] = serde_json::json!("fixed64");
    assert!(produce_joint_candidate(&axes, &wrong_type, run, None).is_err());
    assert!(produce_joint_candidate(&axes, &trace, run, None).is_ok());
}
