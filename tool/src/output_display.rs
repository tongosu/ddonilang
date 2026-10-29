//! A3-P1: presentation only. Never stored in world resources or canonical tapes.
use ddonirang_lang::runtime::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputDisplay {
    pub madi: u64,
    pub line_no: i64,
    pub text: String,
}

pub fn text(value: &Value) -> String {
    // Recursively formats typed numbers even in solver results/record packs.
    // Text, exact rationals and integers keep their original representation.
    crate::ddn_runtime::value_to_display_text(value)
}

fn exact_log_integer(value: &serde_json::Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        // World Value JSON renders Fixed64 log indices as integral floats.
        // Only exact safe integers are accepted; never round an index.
        let number = value.as_f64()?;
        (number.is_finite() && number.fract() == 0.0
            && number.abs() <= 9_007_199_254_740_991.0).then_some(number as i64)
    })
}

fn positions(resources: &serde_json::Value) -> BTreeSet<(u64, i64)> {
    resources.pointer("/value_json/output_log").and_then(|v| v.as_array())
        .into_iter().flatten().filter_map(|row| {
            exact_log_integer(&row["tick"]).and_then(|n| u64::try_from(n).ok())
                .zip(exact_log_integer(&row["line_no"]))
        }).collect()
}

// A3-P1: metadata follows retained world rows, including explicit log clearing.
pub fn retain_current(outputs: &mut Vec<OutputDisplay>, resources: &serde_json::Value) {
    let current = positions(resources);
    let retained: BTreeMap<_, _> = std::mem::take(outputs).into_iter()
        .filter(|row| current.contains(&(row.madi, row.line_no)))
        .map(|row| ((row.madi, row.line_no), row)).collect();
    *outputs = retained.into_values().collect();
}

pub fn restore(payload: &serde_json::Value) -> Result<Vec<OutputDisplay>, String> {
    let Some(display) = payload.get("output_display") else { return Ok(Vec::new()); };
    if display["schema"] != "ddn.output-display.v1"
        || display["rounding"] != "roundTiesToEven"
        || display["fraction_digits"] != 6
        || display["world_hash_participation"] != false
        || display["world_state_hash"] != payload["state_hash"] {
        return Err("E_OUTPUT_DISPLAY_SNAPSHOT: 수 표시 정보의 판 또는 세계 결속이 다릅니다.".into());
    }
    let rows: Vec<OutputDisplay> = serde_json::from_value(display["rows"].clone())
        .map_err(|_| "E_OUTPUT_DISPLAY_SNAPSHOT: 수 표시 줄 정보가 올바르지 않습니다.".to_string())?;
    let resources = payload.get("resources").or_else(|| payload.pointer("/state/resources"))
        .ok_or_else(|| "E_OUTPUT_DISPLAY_SNAPSHOT: 출력 자원이 없습니다.".to_string())?;
    let current = positions(resources);
    let mut seen = BTreeSet::new();
    for row in &rows {
        let key = (row.madi, row.line_no);
        if !current.contains(&key) || !seen.insert(key) {
            return Err("E_OUTPUT_DISPLAY_SNAPSHOT: 표시 줄이 현재 출력과 대응하지 않습니다.".into());
        }
    }
    Ok(rows)
}

pub fn attach(payload: &mut serde_json::Value, outputs: &[OutputDisplay]) {
    if outputs.is_empty() { return; }
    // A3-P1: a long output log must not do a history scan for every row.
    // Later writes to the same logical line win, matching the world log.
    let by_position: BTreeMap<_, _> = outputs.iter()
        .map(|row| ((row.madi, row.line_no), row.text.as_str())).collect();
    let by_line: BTreeMap<_, _> = outputs.iter()
        .map(|row| (row.line_no, row.text.as_str())).collect();
    let texts = if let Some(log) = payload.pointer("/resources/value_json/output_log").and_then(|v| v.as_array()) {
        log.iter().map(|row| {
            exact_log_integer(&row["tick"]).and_then(|n| u64::try_from(n).ok())
                .zip(exact_log_integer(&row["line_no"]))
                .and_then(|key| by_position.get(&key))
                .map(|text| (*text).to_string())
                .unwrap_or_else(|| row["text"].as_str().unwrap_or_default().to_string())
        }).collect::<Vec<_>>()
    } else if let Some(log) = payload["output_log_texts"].as_array() {
        log.iter().enumerate().map(|(index, row)| by_line.get(&(index as i64 + 1))
            .map(|text| (*text).to_string())
            .unwrap_or_else(|| row.as_str().unwrap_or_default().to_string())).collect()
    } else { Vec::new() };
    payload["output_display"] = serde_json::json!({
        "schema": "ddn.output-display.v1",
        "rounding": "roundTiesToEven",
        "fraction_digits": 6,
        "world_hash_participation": false,
        "world_state_hash": payload["state_hash"],
        "rows": outputs,
        "texts": texts,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::Fixed64;

    #[test]
    fn a3_p1_snapshot_display_binding_and_retention() {
        let mut payload = serde_json::json!({"state_hash":"fixed-world", "resources":{"value_json":{"output_log":[
            {"tick":2.0,"line_no":8.0,"text":"raw"}
        ]}}});
        let mut rows = vec![OutputDisplay{madi:2,line_no:8,text:"19.02".into()},
            OutputDisplay{madi:1,line_no:1,text:"old".into()}];
        retain_current(&mut rows, &payload["resources"]);
        assert_eq!(rows.len(),1);
        attach(&mut payload,&rows);
        assert_eq!(restore(&payload).unwrap(),rows);
        payload["state_hash"] = serde_json::json!("different-world");
        assert!(restore(&payload).unwrap_err().contains("E_OUTPUT_DISPLAY_SNAPSHOT"));
        retain_current(&mut rows,&serde_json::json!({"value_json":{"output_log":[]}}));
        assert!(rows.is_empty());
        assert!(restore(&serde_json::json!({"state_hash":"old snapshot"})).unwrap().is_empty());
    }

    #[test]
    fn a3_p1_integral_world_log_indices_and_sparse_lines() {
        let mut payload = serde_json::json!({"resources":{"value_json":{"output_log":[
            {"tick":2.0,"line_no":8.0,"text":"raw"},
            {"tick":2.5,"line_no":9.0,"text":"do not guess"}
        ]}}});
        attach(&mut payload, &[OutputDisplay {madi:2,line_no:8,text:"19.02".into()}]);
        assert_eq!(payload["output_display"]["texts"],serde_json::json!(["19.02","do not guess"]));
        assert_eq!(payload["resources"]["value_json"]["output_log"][0]["text"],"raw");
    }

    #[test]
    fn a3_p1_rounds_typed_numbers_not_text_and_preserves_raw() {
        let value = Fixed64::from_i64(20)
            - Fixed64::parse_decimal_exact("9.8").unwrap()
              * Fixed64::parse_decimal_exact("0.1").unwrap();
        let raw = value.raw_i64();
        assert_eq!(text(&Value::Fixed64(value)), "19.02");
        assert_eq!(value.raw_i64(), raw);
        assert_eq!(text(&Value::String("19.019999999".into())), "19.019999999");
        assert_eq!(text(&Value::List(vec![Value::Fixed64(value), Value::String("19.019999999".into())])),
                   "차림[19.02, 19.019999999]");
        let pack = Value::Pack(BTreeMap::from([
            ("수".into(), Value::Fixed64(value)),
            ("원문".into(), Value::String("19.019999999".into())),
        ]));
        assert_eq!(text(&pack), "(수: 19.02, 원문: 19.019999999)");
        assert_eq!(crate::ddn_runtime::value_to_string(&pack),
            "(수: 19.019999999, 원문: 19.019999999)");
        for (raw, expected) in [(1i64 << 25, "0.007812"), (3i64 << 25, "0.023438"),
                                (-(3i64 << 25), "-0.023438"), (-1, "0")] {
            assert_eq!(text(&Value::Fixed64(Fixed64::from_raw_i64(raw))), expected);
        }
    }
}
