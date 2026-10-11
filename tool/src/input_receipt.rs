//! 실행 입력 영수증. 저장 Tape의 새 형식이 아니라 실제 전달값의 결속이다.
use ddonirang_core::InputSnapshot;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const SNAPSHOT_ENCODING: &str = "ddn.input_snapshot.exact_fixed64.v1";

/// Owner D12: Alpha1 host step; real-time madi rate is an Alpha2 decision.
pub fn alpha_host_dt() -> ddonirang_core::Fixed64 {
    ddonirang_core::Fixed64::from_i64(1)
}

/// Owner A2-D2: host pacing only; never derive world dt from elapsed time.
pub fn alpha2_host_madi_hz() -> u32 {
    60
}

pub fn validate_alpha_host_dt(dt: ddonirang_core::Fixed64) -> Result<(), String> {
    if dt == alpha_host_dt() { Ok(()) } else {
        Err("E_ALPHA_HOST_DT_UNSUPPORTED: 알파1 실행의 호스트 dt는 1입니다. 다른 간격의 입력이나 옛 상태는 실행하지 않습니다".into())
    }
}

/// Existing delivered-snapshot receipt admission, not a successor Tape format.
pub fn parse_receipt_snapshot(value: &Value) -> Result<InputSnapshot, String> {
    let invalid = || "E_INPUT_RECEIPT_INVALID: 입력 영수증의 전달값이 불완전합니다".to_string();
    let snapshot = value.get("input_snapshot").ok_or_else(invalid)?;
    let fields = snapshot.get("input").ok_or_else(invalid)?;
    let dt = parse_recorded_dt(fields)?;
    let input = InputSnapshot {
        tick_id: snapshot.get("madi").and_then(Value::as_u64).ok_or_else(invalid)?,
        dt,
        keys_pressed: fields.get("keys_pressed").and_then(Value::as_u64).ok_or_else(invalid)?,
        last_key_name: fields.get("last_key_name").and_then(Value::as_str).ok_or_else(invalid)?.into(),
        pointer_x_i32: fields.get("pointer_x_i32").and_then(Value::as_i64)
            .and_then(|n| i32::try_from(n).ok()).ok_or_else(invalid)?,
        pointer_y_i32: fields.get("pointer_y_i32").and_then(Value::as_i64)
            .and_then(|n| i32::try_from(n).ok()).ok_or_else(invalid)?,
        rng_seed: fields.get("rng_seed").and_then(Value::as_u64).ok_or_else(invalid)?,
        frame_source: ddonirang_core::InputSource::Person,
        ai_injections: vec![], net_events: vec![],
    };
    // Reject unsupported/noncanonical fields rather than silently dropping them.
    if snapshot_value(&input) != *snapshot {
        return Err("E_INPUT_RECEIPT_UNSUPPORTED: 이 입구가 재현할 수 없는 입력이 포함되어 있습니다".into());
    }
    let expected = value.get("input_snapshot_sha256").and_then(Value::as_str).ok_or_else(invalid)?;
    validate_snapshot_identity(&input, expected)?;
    Ok(input)
}

pub fn record_sha256(inputs: &[InputSnapshot]) -> String {
    let values: Vec<_> = inputs.iter().map(snapshot_value).collect();
    format!("{:x}", Sha256::digest(json!(values).to_string().as_bytes()))
}

pub fn parse_recorded_dt(input: &Value) -> Result<ddonirang_core::Fixed64, String> {
    if input.get("input_encoding").and_then(Value::as_str) != Some(SNAPSHOT_ENCODING) {
        return Err("E_INPUT_RECORD_LEGACY_UNSUPPORTED: 옛 입력 기록은 정확한 dt를 확인할 수 없어 재현하지 않습니다".into());
    }
    let text = input.get("dt").and_then(Value::as_str).ok_or_else(||
        "E_INPUT_RECORD_DT_INVALID: dt는 정확한 정본 글이어야 합니다".to_string())?;
    let value = ddonirang_core::Fixed64::parse_decimal_exact(text).ok_or_else(||
        "E_INPUT_RECORD_DT_INVALID: dt를 손실 없이 읽을 수 없습니다".to_string())?;
    if exact_dt_text(value) != text {
        return Err("E_INPUT_RECORD_DT_NONCANONICAL: dt가 정본 표기가 아닙니다".into());
    }
    Ok(value)
}

/// Q32 분모는 2^32이므로 십진 전개가 32자리 안에서 정확히 끝난다.
/// 화면용 Fixed64 Display(6자리 반올림)는 입력 identity에 사용하지 않는다.
pub fn exact_dt_text(value: ddonirang_core::Fixed64) -> String {
    value.canonical_decimal_text()
}

pub fn receipt(input: &InputSnapshot, reads: &BTreeSet<String>) -> Value {
    json!({
        "input_snapshot": snapshot_value(input),
        "input_snapshot_sha256": snapshot_sha256(input),
        "input_reads": reads,
        "tape_v1_recording_error": validate_tape_v1_reads(reads).err(),
    })
}

/// 공통 실행 직렬화를 사용한다. Fixed64는 정확한 정본 십진 글로 기록한다.
/// 마디와 전달된 모든 입력을 포함하며 빈 입력 필드도 생략하지 않는다.
pub fn snapshot_value(input: &InputSnapshot) -> Value {
    json!({
        "madi": input.tick_id,
        "input": crate::runtime_surface::serialize_input_snapshot(input, None),
    })
}

pub fn snapshot_sha256(input: &InputSnapshot) -> String {
    // Value는 유효한 JSON만 포함하므로 직렬화는 실패하지 않는다.
    let bytes = snapshot_value(input).to_string();
    format!("{:x}", Sha256::digest(bytes.as_bytes()))
}

pub fn validate_snapshot_identity(input: &InputSnapshot, expected: &str) -> Result<(), String> {
    let actual = snapshot_sha256(input);
    if actual == expected {
        Ok(())
    } else {
        Err(format!("E_INPUT_RECEIPT_IDENTITY_MISMATCH: 입력 기록이 다릅니다 기대={expected} 실제={actual}"))
    }
}

/// 실제 접근 기록만 검사한다. 입력에 값이 있다는 이유만으로 거부하지 않는다.
pub fn validate_tape_v1_reads(reads: &BTreeSet<String>) -> Result<(), String> {
    if let Some(field) = reads.iter().find(|field| matches!(field.as_str(),
        "last_key_name" | "pointer_x_i32" | "pointer_y_i32")) {
        Err(format!("E_INPUT_TAPE_V1_UNRECORDED_READ: Tape V1에 담지 못하는 입력을 읽었습니다: {field}"))
    } else {
        Ok(())
    }
}

/// Encode only the existing Tape V1 wire format. Receipts are live execution
/// evidence, not a second file format accepted for replay.
pub fn encode_tape_v1_from_receipts(receipts: &Value, madi_hz: u32) -> Result<Vec<u8>, String> {
    let rows = receipts.as_array().filter(|rows| !rows.is_empty())
        .ok_or_else(|| "E_INPUT_RECEIPT_INVALID: 저장할 실행 입력 영수증이 없습니다".to_string())?;
    let mut masks = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let input = parse_receipt_snapshot(row)?;
        let reads: BTreeSet<String> = serde_json::from_value(row.get("input_reads").cloned()
            .ok_or_else(|| "E_INPUT_RECEIPT_INVALID: 실제 입력 읽기 기록이 없습니다".to_string())?)
            .map_err(|_| "E_INPUT_RECEIPT_INVALID: 입력 읽기 기록이 올바르지 않습니다".to_string())?;
        validate_tape_v1_reads(&reads)?;
        if input.tick_id != index as u64 {
            return Err("E_SAM_RECORD_ORDER_MISMATCH: 처음부터 연속된 마디만 저장할 수 있습니다".into());
        }
        if input.dt != alpha_host_dt() {
            return Err("E_INPUT_TAPE_V1_DT_UNSUPPORTED: Tape V1은 알파1의 고정 dt=1 실행만 저장합니다".into());
        }
        if input.rng_seed != input.tick_id {
            return Err("E_INPUT_TAPE_V1_SEED_UNSUPPORTED: 바꾼 난수 씨앗은 Tape V1만으로 재현할 수 없어 저장하지 않습니다".into());
        }
        let mask = crate::input_tape::runtime_keys_to_tape_mask(input.keys_pressed);
        if crate::input_tape::tape_mask_to_runtime_keys(mask) != input.keys_pressed {
            return Err("E_INPUT_TAPE_V1_KEYS_UNSUPPORTED: Tape V1에 없는 조작 상태가 있습니다".into());
        }
        masks.push(mask);
    }
    crate::input_tape::encode_input_tape(&crate::input_tape::input_tape_from_masks(madi_hz, &masks)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::{Fixed64, InputSource};

    #[test]
    fn alpha2_host_pacing_does_not_change_world_dt() {
        assert_eq!(alpha2_host_madi_hz(), 60);
        assert_eq!(alpha_host_dt(), Fixed64::from_i64(1));
        assert!(validate_alpha_host_dt(Fixed64::from_i64(1)).is_ok());
        assert!(validate_alpha_host_dt(Fixed64::parse_decimal("0.015625").unwrap()).is_err());
    }

    fn input() -> InputSnapshot {
        InputSnapshot { tick_id: 0, dt: Fixed64::parse_decimal("0.015625").unwrap(),
            keys_pressed: 0, last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
            ai_injections: vec![], net_events: vec![], frame_source: InputSource::Person, rng_seed: 0 }
    }

    #[test]
    fn i1_alpha_file_examples_do_not_read_host_dt() {
        // Exact product sources, not rewritten substitutes. The UI inline
        // template/catalog sources are separately checked at their definitions.
        for (path, source) in [
            ("projectile", include_str!("../../solutions/seamgrim_ui_mvp/lessons/rep_phys_projectile_xy_v1/lesson.ddn")),
            ("pendulum", include_str!("../../solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson.ddn")),
            ("grid", include_str!("../../solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn")),
            ("alpha1-fee", include_str!("../../solutions/seamgrim_ui_mvp/alpha1/01_요금_계산기.ddn")),
            ("alpha1-condition", include_str!("../../solutions/seamgrim_ui_mvp/alpha1/02_조건과_반복.ddn")),
            ("alpha1-table", include_str!("../../solutions/seamgrim_ui_mvp/alpha1/03_자동_함수표.ddn")),
            ("alpha1-time", include_str!("../../solutions/seamgrim_ui_mvp/alpha1/04_시간단계_시뮬레이션.ddn")),
        ] {
            let run = |dt: Fixed64| {
                let records: Vec<_> = (0..3).map(|tick| {
                    let mut delivered = input();
                    delivered.tick_id = tick;
                    delivered.rng_seed = tick;
                    delivered.dt = dt;
                    receipt(&delivered, &BTreeSet::new())
                }).collect();
                crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
                    source, path, Some(3), Some("v1-core-v25"), &json!(records), None,
                ).unwrap_or_else(|error| panic!("{path}: {error}"))
            };
            let cli_dt = run(Fixed64::from_i64(1));
            let action_dt = run(Fixed64::from_ratio(1, 64));
            assert_eq!(cli_dt["state_hash"], action_dt["state_hash"], "{path}");
            assert_eq!(cli_dt["output_log_texts"], action_dt["output_log_texts"], "{path}");
            assert_ne!(cli_dt["input_record_sha256"], action_dt["input_record_sha256"], "{path}");
        }
    }

    #[test]
    fn i1_summary_executes_delivered_receipt_not_default_input() {
        let mut delivered = input();
        delivered.last_key_name = "ArrowLeft".into();
        delivered.keys_pressed = 2;
        let record = receipt(&delivered, &BTreeSet::new());
        assert_eq!(parse_receipt_snapshot(&record).unwrap(), delivered);
        let source = "(매마디)마다 { 입력키 보여주기. }.";
        let summary = crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
            source, "lesson.ddn", Some(1), Some("v1-core-v25"), &json!([record]), None,
        ).unwrap();
        assert_eq!(summary["input_record_sha256"], record_sha256(&[delivered.clone()]));
        assert_eq!(summary["input_receipts"][0]["input_snapshot"], snapshot_value(&delivered));
        assert_eq!(summary["input_receipts"][0]["input_reads"], json!(["last_key_name"]));
        assert_eq!(summary["execution_envelope"]["dt"], "0.015625");
        assert_eq!(summary["execution_envelope"]["input_record_sha256"], summary["input_record_sha256"]);
        assert!(summary["output_log_texts"].to_string().contains("ArrowLeft"));
        let mut changed = receipt(&delivered, &BTreeSet::new());
        changed["input_snapshot"]["input"]["last_key_name"] = json!("ArrowRight");
        assert!(parse_receipt_snapshot(&changed).unwrap_err().starts_with("E_INPUT_RECEIPT_IDENTITY_MISMATCH"));
        assert!(crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
            source, "lesson.ddn", Some(2), Some("v1-core-v25"), &json!([receipt(&delivered, &BTreeSet::new())]), None,
        ).unwrap_err().contains("E_INPUT_RECEIPT_SEQUENCE_INVALID"));
    }

    #[test]
    fn i1_summary_uses_same_raw_params_as_vm() {
        use std::collections::BTreeMap;
        use ddonirang_core::Fixed64;
        use ddonirang_lang::runtime::Value as RuntimeValue;
        let source = "채비 { 값:수 := (1) 매김 { 범위: 0..20. 간격: 0.1. }. }. (매마디)마다 { 값 보여주기. }.";
        let params = BTreeMap::from([("값".into(), RuntimeValue::Fixed64(Fixed64::parse_decimal_exact("12.3").unwrap()))]);
        let mut delivered = input();
        delivered.dt = alpha_host_dt();
        let result = crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
            source, "lesson.ddn", Some(1), Some("v1-core-v25"),
            &json!([receipt(&delivered, &BTreeSet::new())]), Some(&params),
        ).unwrap();
        assert!(result["output_log_texts"].to_string().contains("12.3"), "{result}");
        assert_eq!(result["input_receipts"][0]["input_snapshot_sha256"], snapshot_sha256(&delivered));
    }

    #[test]
    fn i1_receipt_replay_distinguishes_absent_unread_and_read_input() {
        for (source, key, expected_reads) in [
            ("(매마디)마다 { 7 보여주기. }.", "", json!([])),
            ("(매마디)마다 { 7 보여주기. }.", "ArrowLeft", json!([])),
            ("(매마디)마다 { 입력키 보여주기. }.", "ArrowLeft", json!(["last_key_name"])),
        ] {
            let mut delivered = input();
            delivered.last_key_name = key.into();
            let records = json!([receipt(&delivered, &BTreeSet::new())]);
            let run = || crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
                source, "lesson.ddn", Some(1), Some("v1-core-v25"), &records, None,
            ).unwrap();
            let first = run();
            let replay = run();
            let from_file_error = crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
                source, "lesson.ddn", Some(1), Some("v1-core-v25"), &first, None,
            ).unwrap_err();
            assert_eq!(first["state_hash"], replay["state_hash"]);
            assert!(from_file_error.starts_with("E_INPUT_RECEIPT_INVALID"));
            assert_eq!(first["input_record_sha256"], replay["input_record_sha256"]);
            assert_eq!(first["input_receipts"][0]["input_reads"], expected_reads);
            assert_eq!(first["input_receipts"][0]["tape_v1_recording_error"].is_null(), key.is_empty() || !source.contains("입력키"));
        }
    }

    #[test]
    fn i1_input_receipt_binds_every_delivered_scalar_and_rejects_changed_record() {
        let original = input();
        let hash = snapshot_sha256(&original);
        assert_eq!(snapshot_value(&original)["input"]["dt"], "0.015625");
        assert_eq!(snapshot_value(&original)["input"]["last_key_name"], "");
        validate_snapshot_identity(&original, &hash).unwrap();
        let mut mutations = vec![original.clone(); 7];
        mutations[0].tick_id = 1;
        mutations[1].keys_pressed = 2;
        mutations[2].last_key_name = "ArrowLeft".into();
        mutations[3].pointer_x_i32 = 1;
        mutations[4].pointer_y_i32 = -1;
        mutations[5].dt = Fixed64::parse_decimal("0.03125").unwrap();
        mutations[6].rng_seed = 1;
        for changed in mutations {
            assert!(validate_snapshot_identity(&changed, &hash).unwrap_err()
                .starts_with("E_INPUT_RECEIPT_IDENTITY_MISMATCH:"));
        }
    }

    #[test]
    fn i1_dt_text_preserves_every_raw_bit_and_negative_fraction() {
        // 1/60은 수학적으로 유한 십진수가 아니지만 실행값은 유한 Q32 값이다.
        let one_sixtieth = Fixed64::from_ratio(1, 60);
        for raw in [0, 1, -1, 67_108_864, one_sixtieth.raw_i64(), i64::MIN, i64::MAX] {
            let value = Fixed64::from_raw_i64(raw);
            let text = exact_dt_text(value);
            assert_eq!(Fixed64::parse_decimal_exact(&text), Some(value), "{raw}: {text}");
            assert_eq!(exact_dt_text(Fixed64::parse_decimal_exact(&text).unwrap()), text);
            assert_eq!(parse_recorded_dt(&json!({"input_encoding": SNAPSHOT_ENCODING, "dt": text})).unwrap(), value);
        }
        let original = input();
        let mut changed = original.clone();
        changed.dt = Fixed64::from_raw_i64(original.dt.raw_i64() + 1);
        assert_ne!(snapshot_sha256(&original), snapshot_sha256(&changed));
    }

    #[test]
    fn d12_alpha_host_rejects_other_dt_without_changing_decimal_conversion() {
        assert!(validate_alpha_host_dt(alpha_host_dt()).is_ok());
        for value in [Fixed64::from_ratio(1, 60), Fixed64::from_ratio(1, 64), Fixed64::from_i64(0)] {
            assert!(validate_alpha_host_dt(value).unwrap_err().starts_with("E_ALPHA_HOST_DT_UNSUPPORTED:"));
            assert_eq!(Fixed64::parse_decimal_exact(&exact_dt_text(value)), Some(value));
        }
    }

    #[test]
    fn i1_old_or_rounded_input_record_is_not_silently_replayed() {
        assert!(parse_recorded_dt(&json!({"dt": "0.016666"})).unwrap_err()
            .starts_with("E_INPUT_RECORD_LEGACY_UNSUPPORTED:"));
        assert!(parse_recorded_dt(&json!({"input_encoding": SNAPSHOT_ENCODING, "dt": "0.016666"})).is_err());
        assert!(parse_recorded_dt(&json!({"input_encoding": SNAPSHOT_ENCODING, "dt": "1.0"})).unwrap_err()
            .starts_with("E_INPUT_RECORD_DT_NONCANONICAL:"));
    }

    #[test]
    fn i1_tape_v1_export_checks_actual_reads_and_preserves_existing_wire() {
        let mut delivered = input();
        delivered.dt = alpha_host_dt();
        delivered.last_key_name = "ArrowLeft".into();
        delivered.pointer_x_i32 = 12;
        delivered.keys_pressed = crate::input_tape::tape_mask_to_runtime_keys(1);
        let rows = json!([receipt(&delivered, &BTreeSet::new())]);
        let bytes = encode_tape_v1_from_receipts(&rows, 60).unwrap();
        assert!(bytes.starts_with(b"DDN_INPUT_TAPE_V1\n"));
        let decoded = crate::input_tape::decode_input_tape(&bytes).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.madi_hz, 60);
        assert_eq!(crate::input_tape::mask_from_bytes(&decoded.records[0].held_mask).unwrap(), 1);
        assert_eq!(crate::input_tape::encode_input_tape(&decoded).unwrap(), bytes);
        let reads = BTreeSet::from(["last_key_name".to_string()]);
        assert!(encode_tape_v1_from_receipts(&json!([receipt(&delivered, &reads)]), 60)
            .unwrap_err().starts_with("E_INPUT_TAPE_V1_UNRECORDED_READ:"));
        delivered.dt = Fixed64::from_ratio(1, 64);
        assert!(encode_tape_v1_from_receipts(&json!([receipt(&delivered, &BTreeSet::new())]), 60)
            .unwrap_err().starts_with("E_INPUT_TAPE_V1_DT_UNSUPPORTED:"));
    }

    #[test]
    fn i1_tape_v1_replay_checks_actual_reads_before_publishing() {
        let tape = crate::input_tape::input_tape_from_masks(60, &[1]).unwrap();
        for body in ["7 보여주기.", "(\"left\") 눌렸나 보여주기.",
            "거짓 그리고 (입력키 == \"ArrowLeft\") 보여주기."] {
            let source = format!("(매마디)마다 {{ {body} }}.");
            let first = crate::runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
                &source, "i1.ddn", Some(1), Some("v1-core-v25"), &tape).unwrap();
            let again = crate::runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
                &source, "i1.ddn", Some(1), Some("v1-core-v25"), &tape).unwrap();
            assert_eq!(first["state_hash"], again["state_hash"]);
        }
        let error = crate::runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
            "(매마디)마다 { 입력키 보여주기. }.", "i1.ddn", Some(1), Some("v1-core-v25"), &tape).unwrap_err();
        assert!(error.contains("E_INPUT_TAPE_V1_UNRECORDED_READ"), "{error}");
        assert!(!error.contains("state_hash"), "실패한 마디 게시 금지: {error}");
    }
}
