use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
use serde_json::Value;

fn persisted_record() -> Value {
    serde_json::json!({
        "type":"record", "version":1,
        "type_identity":{"가로":{"kind":"Named","name":"셈수"}},
        "fields":[{"name":"가로","value":{"type":"fixed64","raw_i64":"42949672960"}}]
    })
}

#[test]
fn vc07_record_codec_identity_and_map_compatibility() {
    use ddonirang_tool::unit_wire::{resource_value_from_detjson as decode, resource_value_to_detjson as encode};
    let raw = persisted_record();
    let value = decode(&raw).unwrap();
    assert_eq!(encode(&value), raw);
    let mut bytes = Vec::new(); value.encode_canon(&mut bytes);
    assert_eq!(&bytes[..2], &[0x0a, 1]);
    for _ in 0..2 {
        let reopened = decode(&serde_json::from_slice(&serde_json::to_vec(&raw).unwrap()).unwrap()).unwrap();
        let mut again = Vec::new(); reopened.encode_canon(&mut again);
        assert_eq!(bytes, again);
        assert_eq!(blake3::hash(&bytes), blake3::hash(&again));
    }
    let map = serde_json::json!({"type":"map","entries":[{
        "key":{"type":"string","value":"가로"},
        "value":{"type":"fixed64","raw_i64":"42949672960"}
    }]});
    let old = decode(&map).unwrap();
    assert!(matches!(old, ddonirang_core::platform::ResourceValue::Map(_)));
    assert_eq!(encode(&old), map);
    let mut actual = Vec::new(); old.encode_canon(&mut actual);
    // Literal predecessor Map encoding: tag, count, string key, raw Fixed64.
    let mut expected = vec![8]; expected.extend_from_slice(&1_u64.to_le_bytes());
    expected.push(4); expected.extend_from_slice(&6_u64.to_le_bytes());
    expected.extend_from_slice("가로".as_bytes()); expected.push(2);
    expected.extend_from_slice(&42949672960_i64.to_le_bytes());
    assert_eq!(actual, expected);
    assert_eq!(blake3::hash(&actual), blake3::hash(&expected));
    assert_ne!(bytes, actual);
}

#[test]
fn vc07_record_wire_rejects_corruption_without_map_fallback() {
    use ddonirang_tool::unit_wire::resource_value_from_detjson as decode;
    let mut invalid = Vec::new();
    let mut value = persisted_record(); value["version"] = serde_json::json!(2); invalid.push(value);
    let mut value = persisted_record(); value["version"] = serde_json::json!(1.0); invalid.push(value);
    let mut value = persisted_record(); value["type"] = serde_json::json!("record-v999"); invalid.push(value);
    let mut value = persisted_record(); value["fields"] = serde_json::json!([]); invalid.push(value);
    let mut value = persisted_record(); let duplicate = value["fields"][0].clone();
    value["fields"].as_array_mut().unwrap().push(duplicate); invalid.push(value);
    let mut value = persisted_record(); value["type_identity"]["가로"]["name"] = serde_json::json!("글"); invalid.push(value);
    let mut value = persisted_record(); value["type_identity"]["가로"]["kind"] = serde_json::json!("Unknown"); invalid.push(value);
    let mut value = persisted_record(); value["fields"][0]["value"] = serde_json::json!({"type":"string","value":"10"}); invalid.push(value);
    let mut value = persisted_record(); value["extra"] = serde_json::json!(true); invalid.push(value);
    for broken in [serde_json::json!({"type":"fixed64","raw_i64":"+42949672960"}),
        serde_json::json!({"type":"bool","value":null}),
        serde_json::json!({"type":"list","items":null})] {
        let mut value = persisted_record(); value["fields"][0]["value"] = broken; invalid.push(value);
    }
    for value in invalid { assert!(decode(&value).is_err(), "accepted corrupt record: {value}"); }
}

#[test]
fn vc07_nested_record_identity_survives_codec_without_host_order() {
    use ddonirang_tool::unit_wire::{resource_value_from_detjson as decode, resource_value_to_detjson as encode};
    let inner = persisted_record();
    let value = serde_json::json!({"type":"record","version":1,
        "type_identity":{"점":{"kind":"Record","fields":inner["type_identity"]},
            "표시":{"kind":"Named","name":"글"}},
        "fields":[{"name":"점","value":inner},
            {"name":"표시","value":{"type":"string","value":"한글😀"}}]});
    let decoded = decode(&value).unwrap();
    assert_eq!(encode(&decoded), value);
    let mut bytes = Vec::new(); decoded.encode_canon(&mut bytes);
    let reopened = decode(&encode(&decoded)).unwrap();
    let mut other = Vec::new(); reopened.encode_canon(&mut other);
    assert_eq!(bytes, other);
    let mut wrong_order = value; wrong_order["fields"].as_array_mut().unwrap().reverse();
    assert!(decode(&wrong_order).is_err(), "canonical field order is explicit, not host order");
}

#[test]
fn vc07_restored_record_callable_and_corrupt_state_atomicity() {
    use ddonirang_core::platform::{InputSnapshot, NuriWorld, ResourceValue};
    use ddonirang_tool::ddn_runtime::{DdnProgram, DdnRunner};
    use ddonirang_tool::unit_wire::{resource_value_from_detjson as decode, resource_value_to_detjson as encode};
    let source = "(가로:셈수) 좌표:묶음씨 := {}.\n(점:좌표) 가로값:셈씨 := 점.가로.\n매마디:움직씨 := { 답 <- (점=점) 가로값. 답 보여주기. }";
    let make_runner = || DdnRunner::new(DdnProgram::from_v25_source(source, "lesson.ddn").unwrap(), "매마디");
    let input = InputSnapshot {
        tick_id: 0, dt: ddonirang_core::Fixed64::from_i64(1), keys_pressed: 0,
        last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
        ai_injections: Vec::new(), net_events: Vec::new(),
        frame_source: ddonirang_core::platform::InputSource::Person, rng_seed: 0,
    };
    let value = decode(&persisted_record()).unwrap();
    let saved = serde_json::to_vec(&encode(&value)).unwrap();
    let reopened = decode(&serde_json::from_slice(&saved).unwrap()).unwrap();
    let mut world = NuriWorld::new(); world.set_resource_value("점".into(), reopened.clone());
    let old_hash = world.state_hash();
    let output = make_runner().run_update(&world, &input, &Default::default()).unwrap();
    assert!(matches!(output.resources.get("답"), Some(ddonirang_lang::Value::Fixed64(n)) if n.int_part() == 10));
    assert_eq!(world.state_hash(), old_hash, "runner only produces a candidate");
    let mut foreign = persisted_record();
    foreign["type_identity"]["가로"]["name"] = serde_json::json!("글");
    foreign["fields"][0]["value"] = serde_json::json!({"type":"string","value":"한글😀"});
    world.set_resource_value("점".into(), decode(&foreign).unwrap());
    let foreign_hash = world.state_hash();
    let error = make_runner().run_update(&world, &input, &Default::default()).err().unwrap();
    assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"), "valid foreign record type was accepted: {error}");
    assert_eq!(world.state_hash(), foreign_hash);
    let ResourceValue::Record { field_types, fields } = &reopened else { panic!("lost record identity") };
    let mut corrupted = fields.clone(); corrupted.clear();
    world.set_resource_value("점".into(), ResourceValue::Record { field_types: field_types.clone(), fields: corrupted });
    let old_hash = world.state_hash();
    let mut runner = make_runner();
    let error = runner.run_update(&world, &input, &Default::default()).err().unwrap();
    assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"), "{error}");
    assert_eq!(world.state_hash(), old_hash);
    let map = serde_json::json!({"type":"map","entries":[{"key":{"type":"string","value":"가로"},"value":{"type":"fixed64","raw_i64":"42949672960"}}]});
    world.set_resource_value("점".into(), decode(&map).unwrap());
    let error = runner.run_update(&world, &input, &Default::default()).err().unwrap();
    assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"), "Map shape became a record: {error}");
}

#[test]
fn vc07_known_invalid_field_expression_is_rejected_before_execution() {
    let source = include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/vc07_field_infix_type.ddn");
    let error = ddonirang_tool::ddn_runtime::DdnProgram::from_v25_source(source, "lesson.ddn")
        .err().expect("known invalid operands in Product fields must be rejected before execution");
    assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"), "{error}");
}

#[test]
fn vc07_persisted_product_remains_a_record_on_the_next_madi() {
    let source = include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/vc07_persisted.ddn");
    let result = run_summary_from_supported_source_profile(source, "lesson.ddn", Some(2), Some("v1-core-v25"))
        .expect("typed Product must survive the committed-state boundary");
    assert_eq!(result["output_log_texts"], serde_json::json!(["10", "10"]));
}

#[test]
fn vc07_actual_source_shared_runtime_matrix() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../pack/v1_core_supported_ingress_w1");
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(root.join("manifest.detjson")).unwrap()).unwrap();
    let mut states = std::collections::BTreeMap::new();
    for case in manifest["vc07_cases"].as_array().unwrap() {
        let name = case[0].as_str().unwrap();
        let source = std::fs::read_to_string(root.join("fixtures").join(name)).unwrap();
        let run = || run_summary_from_supported_source_profile(&source, "lesson.ddn", Some(1), Some("v1-core-v25"));
        let first = run();
        assert_eq!(first, run(), "{name}: repeat");
        if case[1].is_null() {
            let error = first.expect_err(name);
            assert!(error.contains(case[2].as_str().unwrap()), "{name}: {error}");
            let failure: Value = serde_json::from_str(&error).unwrap();
            assert_eq!(failure["code"], case[2], "{name}: structured failure code");
            if name == "vc07_unicode_pin.ddn" {
                assert!(failure["message"].as_str().unwrap().contains("(2:9)"), "{error}");
            }
            for field in ["output_log_texts", "state_hash", "trace_hash", "work_cost_receipt"] {
                assert!(failure.get(field).is_none(), "{name}: partial {field}");
            }
        } else {
            let result = first.unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(result["output_log_texts"], case[1], "{name}: output");
            states.insert(name.to_owned(), result["state_hash"].clone());
            let mut parsed = ddonirang_lang::parse_v25(&source, "lesson.ddn").unwrap();
            ddonirang_lang::canonicalize(&mut parsed).unwrap();
            let canonical = ddonirang_lang::normalize(&parsed, ddonirang_lang::NormalizationLevel::N1);
            let repeated = run_summary_from_supported_source_profile(&canonical, "lesson.ddn", Some(1), Some("v1-core-v25")).unwrap();
            assert_eq!(result["output_log_texts"], repeated["output_log_texts"], "{name}: canon");
            assert_eq!(result["state_hash"], repeated["state_hash"], "{name}: canon state");
        }
    }
    assert_eq!(states["vc07_record.ddn"], states["vc07_reordered.ddn"], "authored field order is not state identity");
}
