use ddonirang_core::platform::{ResourceMapEntry, ResourceValue};
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue};
use serde_json::{json, Value as JsonValue};

use crate::fixed64_boundary::Fixed64FloatBoundary;

pub const RESOURCE_SNAPSHOT_SCHEMA_V2: &str = "ddn.resource_snapshot.v2";
pub const UNIT_VALUE_WIRE_SCHEMA_V2: &str = "ddn.unit_value_wire.v2";

pub fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    match value {
        ResourceValue::None => json!({ "type": "none" }),
        ResourceValue::Bool(v) => json!({ "type": "bool", "value": v }),
        ResourceValue::Fixed64(v) => {
            json!({ "type": "fixed64", "raw_i64": v.raw_i64().to_string() })
        }
        ResourceValue::Unit(v) => json!({
            "type": "unit",
            "schema": UNIT_VALUE_WIRE_SCHEMA_V2,
            "raw_i64": v.value.raw_i64().to_string(),
            "dimension": {
                "schema": UnitDim::WIRE_SCHEMA_V2,
                "axis_order": UnitDim::AXIS_ORDER_V2,
                "exponents": v.dim.exponents(),
            },
        }),
        ResourceValue::String(v) => json!({ "type": "string", "value": v }),
        ResourceValue::ResourceHandle(v) => {
            json!({ "type": "handle", "value": handle_to_string(*v) })
        }
        ResourceValue::List(items) => json!({
            "type": "list",
            "items": items.iter().map(resource_value_to_detjson).collect::<Vec<_>>(),
        }),
        ResourceValue::Set(items) => json!({
            "type": "set",
            "items": items.values().map(resource_value_to_detjson).collect::<Vec<_>>(),
        }),
        ResourceValue::Map(entries) => {
            let rows: Vec<JsonValue> = entries
                .values()
                .map(|entry| {
                    json!({
                        "key": resource_value_to_detjson(&entry.key),
                        "value": resource_value_to_detjson(&entry.value),
                    })
                })
                .collect();
            json!({ "type": "map", "entries": rows })
        }
    }
}

pub fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    let JsonValue::Object(obj) = payload else {
        return Err("value_det 형식 오류(object 아님)".to_string());
    };
    let kind = obj.get("type").and_then(JsonValue::as_str).unwrap_or_default();
    match kind {
        "none" => Ok(ResourceValue::None),
        "bool" => Ok(ResourceValue::Bool(
            obj.get("value").and_then(JsonValue::as_bool).unwrap_or(false),
        )),
        "fixed64" => {
            let value = obj
                .get("raw_i64")
                .and_then(JsonValue::as_str)
                .and_then(|raw| raw.parse::<i64>().ok())
                .map(Fixed64::from_raw_i64)
                .or_else(|| obj.get("value").and_then(json_to_fixed64))
                .ok_or_else(|| "fixed64 value 파싱 실패".to_string())?;
            Ok(ResourceValue::Fixed64(value))
        }
        "unit" => parse_unit_value_v2(obj),
        "string" => Ok(ResourceValue::String(
            obj.get("value")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string(),
        )),
        "handle" => {
            let raw = obj
                .get("value")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| "handle value 누락".to_string())
                .and_then(parse_handle_string)?;
            Ok(ResourceValue::ResourceHandle(ResourceHandle::from_raw(raw)))
        }
        "list" => {
            let items = obj
                .get("items")
                .and_then(JsonValue::as_array)
                .cloned()
                .unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(&item)?);
            }
            Ok(ResourceValue::List(out))
        }
        "set" => {
            let items = obj
                .get("items")
                .and_then(JsonValue::as_array)
                .cloned()
                .unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(&item)?);
            }
            Ok(ResourceValue::set_from_values(out))
        }
        "map" => {
            let rows = obj
                .get("entries")
                .and_then(JsonValue::as_array)
                .cloned()
                .unwrap_or_default();
            let mut entries = Vec::with_capacity(rows.len());
            for row in rows {
                let JsonValue::Object(row_obj) = row else {
                    return Err("map entry 형식 오류".to_string());
                };
                let key = row_obj
                    .get("key")
                    .ok_or_else(|| "map entry key 누락".to_string())
                    .and_then(resource_value_from_detjson)?;
                let value = row_obj
                    .get("value")
                    .ok_or_else(|| "map entry value 누락".to_string())
                    .and_then(resource_value_from_detjson)?;
                entries.push(ResourceMapEntry { key, value });
            }
            Ok(ResourceValue::map_from_entries(entries))
        }
        _ => Err(format!("지원하지 않는 value_det type: {kind}")),
    }
}

fn parse_unit_value_v2(
    obj: &serde_json::Map<String, JsonValue>,
) -> Result<ResourceValue, String> {
    let allowed_keys = ["type", "schema", "raw_i64", "dimension"];
    if let Some(unknown) = obj
        .keys()
        .find(|key| !allowed_keys.contains(&key.as_str()))
    {
        return Err(format!("E_UNIT_WIRE_UNKNOWN_FIELD: {unknown}"));
    }
    let schema = obj.get("schema").and_then(JsonValue::as_str);
    if schema != Some(UNIT_VALUE_WIRE_SCHEMA_V2) {
        return Err(format!(
            "E_UNIT_WIRE_VERSION_MISMATCH: expected {UNIT_VALUE_WIRE_SCHEMA_V2}, got {}",
            schema.unwrap_or("<missing>")
        ));
    }
    let raw = obj
        .get("raw_i64")
        .and_then(JsonValue::as_str)
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or_else(|| "unit raw_i64 파싱 실패".to_string())?;
    let dimension = obj
        .get("dimension")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "E_UNIT_DIMENSION_WIRE_INVALID: object가 필요합니다".to_string())?;
    let allowed_dimension_keys = ["schema", "axis_order", "exponents"];
    if let Some(unknown) = dimension
        .keys()
        .find(|key| !allowed_dimension_keys.contains(&key.as_str()))
    {
        return Err(format!("E_UNIT_DIMENSION_WIRE_UNKNOWN_FIELD: {unknown}"));
    }
    let dimension_schema = dimension.get("schema").and_then(JsonValue::as_str);
    if dimension_schema != Some(UnitDim::WIRE_SCHEMA_V2) {
        return Err(format!(
            "E_UNIT_DIMENSION_WIRE_VERSION_MISMATCH: expected {}, got {}",
            UnitDim::WIRE_SCHEMA_V2,
            dimension_schema.unwrap_or("<missing>")
        ));
    }
    let axis_order = dimension
        .get("axis_order")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "E_UNIT_DIMENSION_AXIS_ORDER_MISSING".to_string())?;
    let actual_axis_order = axis_order
        .iter()
        .map(|axis| axis.as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    if actual_axis_order.as_slice() != UnitDim::AXIS_ORDER_V2 {
        return Err(format!(
            "E_UNIT_DIMENSION_AXIS_ORDER_MISMATCH: expected {:?}, got {:?}",
            UnitDim::AXIS_ORDER_V2,
            actual_axis_order
        ));
    }
    let dimensions = dimension
        .get("exponents")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "E_UNIT_DIMENSION_EXPONENTS_MISSING".to_string())?;
    if dimensions.len() != UnitDim::AXIS_COUNT {
        return Err(format!(
            "E_UNIT_DIMENSION_AXIS_COUNT_MISMATCH: expected {}, got {}",
            UnitDim::AXIS_COUNT,
            dimensions.len()
        ));
    }
    let mut exponents = [0i8; UnitDim::AXIS_COUNT];
    for (index, value) in dimensions.iter().enumerate() {
        exponents[index] = value
            .as_i64()
            .and_then(|row| i8::try_from(row).ok())
            .ok_or_else(|| format!("E_UNIT_DIMENSION_EXPONENT_INVALID: index {index}"))?;
    }
    Ok(ResourceValue::Unit(UnitValue {
        value: Fixed64::from_raw_i64(raw),
        dim: UnitDim::from_exponents(exponents),
    }))
}

fn json_to_fixed64(value: &JsonValue) -> Option<Fixed64> {
    match value {
        JsonValue::Number(n) => n.as_f64().map(Fixed64::from_f64_lossy),
        JsonValue::String(s) => Fixed64::parse_decimal(s),
        JsonValue::Object(obj) => obj
            .get("raw_i64")
            .and_then(JsonValue::as_str)
            .and_then(|raw| raw.parse::<i64>().ok())
            .map(Fixed64::from_raw_i64),
        _ => None,
    }
}

fn parse_handle_string(text: &str) -> Result<u64, String> {
    let trimmed = text.trim();
    let without_handle = trimmed.strip_prefix("handle:").unwrap_or(trimmed);
    let hex = without_handle.strip_prefix("자원#").unwrap_or(without_handle);
    u64::from_str_radix(hex, 16).map_err(|_| format!("handle 파싱 실패: {text}"))
}

fn handle_to_string(handle: ResourceHandle) -> String {
    format!("자원#{:016x}", handle.raw())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::Unit;

    fn mol_value() -> ResourceValue {
        ResourceValue::Unit(UnitValue::new(Fixed64::from_i64(2), Unit::Mole))
    }

    #[test]
    fn common_product_unit_wire_roundtrips_explicit_v2() {
        let encoded = resource_value_to_detjson(&mol_value());
        assert_eq!(encoded["schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(encoded["dimension"]["schema"], UnitDim::WIRE_SCHEMA_V2);
        assert_eq!(
            encoded["dimension"]["axis_order"],
            json!(UnitDim::AXIS_ORDER_V2)
        );
        assert_eq!(
            encoded["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            resource_value_from_detjson(&encoded).expect("v2 roundtrip"),
            mol_value()
        );
    }

    #[test]
    fn common_product_unit_wire_rejects_unversioned_and_unknown_versions() {
        let unversioned = json!({
            "type": "unit",
            "raw_i64": "8589934592",
            "dimension": [0, 0, 0, 0, 0, 0, 0, 0],
        });
        assert!(resource_value_from_detjson(&unversioned)
            .expect_err("unversioned v1")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut unknown = resource_value_to_detjson(&mol_value());
        unknown["schema"] = json!("ddn.unit_value_wire.v3");
        assert!(resource_value_from_detjson(&unknown)
            .expect_err("unknown v3")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));
    }

    #[test]
    fn common_product_unit_wire_rejects_axis_reorder_and_missing_ninth_axis() {
        let mut reordered = resource_value_to_detjson(&mol_value());
        reordered["dimension"]["axis_order"][0] = json!("time");
        reordered["dimension"]["axis_order"][1] = json!("length");
        assert!(resource_value_from_detjson(&reordered)
            .expect_err("reordered axes")
            .contains("E_UNIT_DIMENSION_AXIS_ORDER_MISMATCH"));

        let mut missing = resource_value_to_detjson(&mol_value());
        missing["dimension"]["exponents"]
            .as_array_mut()
            .expect("exponents")
            .pop();
        assert!(resource_value_from_detjson(&missing)
            .expect_err("missing ninth axis")
            .contains("E_UNIT_DIMENSION_AXIS_COUNT_MISMATCH"));
    }
}
