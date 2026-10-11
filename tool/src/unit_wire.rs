use ddonirang_core::platform::{ResourceMapEntry, ResourceValue};
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue};
use serde_json::{json, Value as JsonValue};

use crate::fixed64_boundary::reject_binary_json_number;

// RD-674/RD-682/RD-683: one shared issuer/verifier for all product surfaces.
mod tensor_frame;
pub(crate) mod project_runtime_attachment;
mod complete_state_frame;
mod complete_state_tables;
mod saved_resource_value;
pub(crate) use complete_state_frame::{check_complete_state_frame_v1, CompleteStateFrameV1, CompleteStateValueTablesV1};

#[cfg(test)]
mod tensor_selection_tests {
    use super::*;
    use ddonirang_lang::{ast::TypeRef, runtime::Value};

    fn matrix() -> ddonirang_lang::runtime::TensorValue {
        tensor_frame::encode_typed_tensor(&TypeRef::Named("셈수".into()), &[2,3],
            &(0..6).map(Fixed64::from_i64).collect::<Vec<_>>()).unwrap().into_runtime().unwrap()
    }

    #[test]
    fn rd388_axis_removal_ranges_and_trailing_axes_are_exact() {
        let old = matrix();
        let Value::Tensor(row) = tensor_select(&old, &[TensorSelector::Index(1)]).unwrap() else { panic!() };
        assert_eq!(row.shape(), &[3]);
        assert_eq!(row.elements(), &(3..6).map(Fixed64::from_i64).collect::<Vec<_>>());
        assert_eq!(tensor_select(&old, &[TensorSelector::Index(1),TensorSelector::Index(2)]).unwrap(), Value::Fixed64(Fixed64::from_i64(5)));
        let Value::Tensor(column) = tensor_select(&old, &[TensorSelector::Full,TensorSelector::Range { start: 1, end: 2 }]).unwrap() else { panic!() };
        assert_eq!(column.shape(), &[2,1]);
        assert_eq!(column.elements(), &[Fixed64::from_i64(1),Fixed64::from_i64(4)]);
        let Value::Tensor(empty) = tensor_select(&old, &[TensorSelector::Range { start: 2, end: 2 }]).unwrap() else { panic!() };
        assert_eq!(empty.shape(), &[0,3]);
        assert!(empty.elements().is_empty());
        for selectors in [vec![],vec![TensorSelector::Index(2)],vec![TensorSelector::Full;3],vec![TensorSelector::Range {start:2,end:1}]] {
            assert!(tensor_select(&old,&selectors).is_err());
        }
    }

    #[test]
    fn rd391_replacement_is_persistent_and_never_broadcasts() {
        let old = matrix();
        let before = old.saved_bytes().to_vec();
        let updated = tensor_replace(&old, &[TensorSelector::Index(1),TensorSelector::Index(2)], &Value::Fixed64(Fixed64::from_i64(9))).unwrap();
        assert_eq!(updated.shape(), old.shape());
        assert_eq!(updated.elements()[5],Fixed64::from_i64(9));
        assert_eq!(old.saved_bytes(),before);
        assert!(tensor_replace(&old,&[TensorSelector::Index(1)],&Value::Fixed64(Fixed64::from_i64(9))).is_err());
        let wrong = tensor_frame::encode_typed_tensor(&TypeRef::Named("셈수".into()), &[1,3], &[Fixed64::ZERO;3]).unwrap().into_runtime().unwrap();
        assert!(tensor_replace(&old,&[TensorSelector::Index(1)],&Value::Tensor(wrong)).is_err());
        let row = tensor_frame::encode_typed_tensor(&TypeRef::Named("셈수".into()), &[3], &[Fixed64::from_i64(8);3]).unwrap().into_runtime().unwrap();
        let updated = tensor_replace(&old,&[TensorSelector::Index(1)],&Value::Tensor(row)).unwrap();
        assert_eq!(updated.elements(), &[Fixed64::ZERO,Fixed64::from_i64(1),Fixed64::from_i64(2),Fixed64::from_i64(8),Fixed64::from_i64(8),Fixed64::from_i64(8)]);
        assert_eq!(old.saved_bytes(),before);
    }
}

// RD-388/391: selectors are resolved once per axis, without consuming input
// or mutating the admitted value. Parser keeps source bracket boundaries.
#[derive(Debug, Clone, Copy)]
pub(crate) enum TensorSelector {
    Index(u64),
    Range { start: u64, end: u64 },
    Full,
}

fn tensor_selection_plan(value: &ddonirang_lang::runtime::TensorValue, selectors: &[TensorSelector]) -> Result<(Vec<u64>, Vec<usize>), String> {
    verify_runtime_tensor(value)?;
    if selectors.is_empty() || selectors.len() > value.shape().len() {
        return Err("E_TENSOR_SELECTION: 비어 있거나 축 수를 넘는 선택입니다".into());
    }
    let mut axes = Vec::new();
    let mut shape = Vec::new();
    for (axis, extent) in value.shape().iter().copied().enumerate() {
        let (start, length, retained) = match selectors.get(axis).copied().unwrap_or(TensorSelector::Full) {
            TensorSelector::Index(i) if i < extent => (i, 1, false),
            TensorSelector::Range { start, end } if start <= end && end <= extent => (start, end - start, true),
            TensorSelector::Full => (0, extent, true),
            _ => return Err("E_TENSOR_SELECTION: 축 선택이 범위를 벗어났습니다".into()),
        };
        axes.push((start, length, retained));
        if retained { shape.push(length); }
    }
    // Zero extents short-circuit before stride arithmetic and enumeration.
    // [0, u64::MAX] is a legitimate empty shape, not an allocation request.
    if shape.contains(&0) { return Ok((shape, Vec::new())); }
    let count = shape.iter().try_fold(1u64, |n, d| n.checked_mul(*d))
        .ok_or("E_TENSOR_RESOURCE_LIMIT: 선택 크기 초과")?;
    let mut offsets = Vec::new();
    offsets.try_reserve_exact(usize::try_from(count).map_err(|_| "E_TENSOR_RESOURCE_LIMIT: 선택 크기 초과")?)
        .map_err(|_| "E_TENSOR_RESOURCE_LIMIT: 선택 공간 부족")?;
    for flat in 0..count {
        let mut remainder = flat;
        let mut coordinates = vec![0u64; axes.len()];
        for axis in (0..axes.len()).rev() {
            let (start, length, retained) = axes[axis];
            coordinates[axis] = start + if retained { let n = remainder % length; remainder /= length; n } else { 0 };
        }
        let offset = coordinates.iter().zip(value.shape()).try_fold(0u64, |n, (c, d)| n.checked_mul(*d)?.checked_add(*c))
            .ok_or("E_TENSOR_SELECTION: 선택 위치 초과")?;
        let offset = usize::try_from(offset).map_err(|_| "E_TENSOR_SELECTION: 선택 위치 초과")?;
        if offset >= value.elements().len() { return Err("E_TENSOR_SELECTION: 선택 위치 초과".into()); }
        offsets.push(offset);
    }
    Ok((shape, offsets))
}

pub(crate) fn tensor_select(value: &ddonirang_lang::runtime::TensorValue, selectors: &[TensorSelector]) -> Result<ddonirang_lang::runtime::Value, String> {
    use ddonirang_lang::runtime::Value;
    let (shape, offsets) = tensor_selection_plan(value, selectors)?;
    if shape.is_empty() { return Ok(Value::Fixed64(value.elements()[offsets[0]])); }
    let elements: Vec<_> = offsets.into_iter().map(|i| value.elements()[i]).collect();
    tensor_frame::encode_typed_tensor(&ddonirang_lang::ast::TypeRef::Named("셈수".into()), &shape, &elements)
        .and_then(|v| v.into_runtime()).map(Value::Tensor)
        .map_err(|e| format!("E_TENSOR_SELECTION: 선택값 검증 실패: {e:?}"))
}

pub(crate) fn tensor_replace(value: &ddonirang_lang::runtime::TensorValue, selectors: &[TensorSelector], replacement: &ddonirang_lang::runtime::Value) -> Result<ddonirang_lang::runtime::TensorValue, String> {
    use ddonirang_lang::runtime::Value;
    let (shape, offsets) = tensor_selection_plan(value, selectors)?;
    let supplied = match replacement {
        Value::Fixed64(v) if shape.is_empty() => vec![*v],
        Value::Tensor(v) if !shape.is_empty() && v.shape() == shape => {
            verify_runtime_tensor(v)?;
            v.elements().to_vec()
        }
        _ => return Err("E_TENSOR_REPLACEMENT: 셀은 셈수, 슬라이스는 정확히 같은 모양의 텐서여야 합니다".into()),
    };
    let mut elements = value.elements().to_vec();
    for (offset, element) in offsets.into_iter().zip(supplied) { elements[offset] = element; }
    tensor_frame::encode_typed_tensor(&ddonirang_lang::ast::TypeRef::Named("셈수".into()), value.shape(), &elements)
        .and_then(|v| v.into_runtime()).map_err(|e| format!("E_TENSOR_REPLACEMENT: 교체값 검증 실패: {e:?}"))
}

pub(crate) fn tensor_from_collection(kind: &ddonirang_lang::ast::TypeRef, shape: &[u64], input: &ddonirang_lang::runtime::Value) -> Result<ddonirang_lang::runtime::TensorValue, String> {
    tensor_frame::admit_collection(kind, shape, input).and_then(|v| v.into_runtime())
        .map_err(|e| match e {
            tensor_frame::TensorFrameError::ResourceLimit => "E_TENSOR_RESOURCE_LIMIT: Tensor 제품 자원 한도 초과".into(),
            _ => format!("E_TENSOR_ADMISSION: Tensor 입력 검증 실패: {e:?}"),
        })
}
pub(crate) fn verify_runtime_tensor(value: &ddonirang_lang::runtime::TensorValue) -> Result<(), String> {
    tensor_frame::verify_runtime(value).map_err(|e| format!("E_TENSOR_CARRIER: Tensor carrier 검증 실패: {e:?}"))
}
pub(crate) fn preflight_tensor_collection(shape: &[u64]) -> Result<(), String> {
    tensor_frame::preflight_collection(shape)
        .map_err(|e| format!("E_TENSOR_RESOURCE_LIMIT: 차림 구체화 한도 초과: {e:?}"))
}
pub(crate) fn tensor_to_resource(value: &ddonirang_lang::runtime::TensorValue) -> Result<ResourceValue, String> {
    verify_runtime_tensor(value)?;
    Ok(ResourceValue::BasicTensorV1(value.saved_bytes()[1..].to_vec()))
}
pub(crate) fn tensor_from_carrier(carrier: &[u8]) -> Result<ddonirang_lang::runtime::TensorValue, String> {
    tensor_frame::from_carrier(carrier).and_then(|v| v.into_runtime()).map_err(|e| format!("Tensor 복원 검증 실패: {e:?}"))
}

pub(crate) fn parse_snapshot_json(raw: &str) -> Result<JsonValue, String> {
    use serde::de::{DeserializeSeed, Visitor, MapAccess, SeqAccess, Error};
    struct Seed(bool);
    impl<'de> DeserializeSeed<'de> for Seed {
        type Value = JsonValue;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<JsonValue, D::Error> {
            d.deserialize_any(self)
        }
    }
    impl<'de> Visitor<'de> for Seed {
        type Value = JsonValue;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("저장 스냅샷") }
        fn visit_bool<E: Error>(self, v: bool) -> Result<JsonValue,E> { Ok(json!(v)) }
        fn visit_i64<E: Error>(self, v: i64) -> Result<JsonValue,E> { Ok(json!(v)) }
        fn visit_u64<E: Error>(self, v: u64) -> Result<JsonValue,E> { Ok(json!(v)) }
        fn visit_f64<E: Error>(self, v: f64) -> Result<JsonValue,E> { Ok(json!(v)) }
        fn visit_str<E: Error>(self, v: &str) -> Result<JsonValue,E> { Ok(json!(v)) }
        fn visit_string<E: Error>(self, v: String) -> Result<JsonValue,E> { Ok(JsonValue::String(v)) }
        fn visit_unit<E: Error>(self) -> Result<JsonValue,E> { Ok(JsonValue::Null) }
        fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<JsonValue,S::Error> {
            let mut out = Vec::new();
            while let Some(v) = seq.next_element_seed(Seed(self.0))? { out.push(v); }
            Ok(JsonValue::Array(out))
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<JsonValue,M::Error> {
            let mut keys = Vec::new();
            let mut obj = serde_json::Map::new();
            let mut tensor = false;
            while let Some(key) = map.next_key::<String>()? {
                // A repeated resource name must not erase an earlier value
                // before its typed admission (including a Tensor) is checked.
                if obj.contains_key(&key) {
                    return Err(M::Error::custom("저장 자원 키 중복"));
                }
                let value = map.next_value_seed(Seed(self.0 || key == "value_det"))?;
                tensor |= self.0 && key == "type" && value.as_str() == Some("tensor_v1");
                keys.push(key.clone()); obj.insert(key, value);
            }
            let value = JsonValue::Object(obj);
            if tensor {
                tensor_frame::validate_transport_keys(keys.iter().map(String::as_str))
                    .map_err(|_| M::Error::custom("Tensor 키 중복 또는 순서 오류"))?;
                tensor_frame::transport_from_value(&value)
                    .map_err(|e| M::Error::custom(format!("Tensor 저장 검증 실패: {e:?}")))?;
            }
            Ok(value)
        }
    }
    // Existing snapshot_v2 host-neutral ceiling (RD-668), not an adapter limit.
    if raw.len() > 2_000_000 { return Err("저장 스냅샷 크기 상한 초과".into()); }
    let mut decoder = serde_json::Deserializer::from_str(raw);
    let value = Seed(false).deserialize(&mut decoder).map_err(|e| e.to_string())?;
    decoder.end().map_err(|e| e.to_string())?;
    Ok(value)
}

pub const RESOURCE_SNAPSHOT_SCHEMA_V2: &str = "ddn.resource_snapshot.v2";
pub const UNIT_VALUE_WIRE_SCHEMA_V2: &str = "ddn.unit_value_wire.v2";

#[cfg(test)]
mod alpha3_tensor_snapshot_tests {
    use super::*;
    use ddonirang_lang::{ast::TypeRef,runtime::Value};

    #[test]
    fn raw_snapshot_rejects_shadowed_tensor_before_json_collapse() {
        let tensor = tensor_from_collection(&TypeRef::Named("셈수".into()), &[2],
            &Value::List(vec![Value::Fixed64(Fixed64::from_i64(1)),Value::Fixed64(Fixed64::from_i64(2))])).unwrap();
        let leaf = resource_value_to_detjson(&tensor_to_resource(&tensor).unwrap()).to_string();
        let raw = format!("{{\"resources\":{{\"value_det\":{{\"판\":{leaf}}}}}}}");
        assert_eq!(parse_snapshot_json(&raw).unwrap(),serde_json::from_str::<JsonValue>(&raw).unwrap());
        for candidate in [
            format!("{{\"resources\":{{\"value_det\":{{\"판\":{leaf},\"판\":{{\"type\":\"none\"}}}}}}}}"),
            format!("{{\"resources\":{{\"value_det\":{{\"판\":{leaf}}}}},\"resources\":{{}}}}"),
            raw.replacen(&leaf,&leaf.replacen("{","{\"version\":1,",1),1),
            raw.replacen("\"carrier_sha256\":","\"carrier_sha256\":\"bad\",\"shadow\":",1),
            format!("{raw} false"),
        ] {
            assert!(parse_snapshot_json(&candidate).is_err(), "must reject raw: {candidate}");
        }
        // Same guard applies when Tensor is nested in an ordinary collection.
        let nested = format!("{{\"resources\":{{\"value_det\":{{\"목록\":{{\"type\":\"list\",\"items\":[{leaf}]}}}}}}}}");
        assert!(parse_snapshot_json(&nested).is_ok());
        let ambiguous = nested.replacen(&leaf,&leaf.replacen("{","{\"version\":1,",1),1);
        assert!(parse_snapshot_json(&ambiguous).is_err());
    }
}

pub fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    match value {
        ResourceValue::BasicTensorV1(carrier) => tensor_frame::transport_value(
            &tensor_frame::from_carrier(carrier).expect("admitted Tensor resource")),
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
        "tensor_v1" => tensor_frame::transport_from_value(payload)
            .map(|v| ResourceValue::BasicTensorV1(v.saved_bytes()[1..].to_vec()))
            .map_err(|e| format!("Tensor 저장 투영 검증 실패: {e:?}")),
        "none" => Ok(ResourceValue::None),
        "bool" => Ok(ResourceValue::Bool(
            obj.get("value").and_then(JsonValue::as_bool)
                .ok_or_else(|| "bool value 형식 오류: 불리언이 필요합니다".to_string())?,
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
                .ok_or_else(|| "string value 형식 오류: 글이 필요합니다".to_string())?
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
                .ok_or_else(|| "list items 형식 오류: 목록이 필요합니다".to_string())?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(item)?);
            }
            Ok(ResourceValue::List(out))
        }
        "set" => {
            let items = obj
                .get("items")
                .and_then(JsonValue::as_array)
                .ok_or_else(|| "set items 형식 오류: 목록이 필요합니다".to_string())?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(item)?);
            }
            Ok(ResourceValue::set_from_values(out))
        }
        "map" => {
            let rows = obj
                .get("entries")
                .and_then(JsonValue::as_array)
                .ok_or_else(|| "map entries 형식 오류: 목록이 필요합니다".to_string())?;
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

pub(crate) fn json_to_fixed64(value: &JsonValue) -> Option<Fixed64> {
    match value {
        JsonValue::Number(n) => reject_binary_json_number(n),
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

    #[test]
    fn saved_value_fields_are_required_never_defaulted() {
        for (kind,field,valid) in [("bool","value",json!(false)),("string","value",json!("")),
            ("list","items",json!([])),("set","items",json!([])),("map","entries",json!([]))] {
            let mut payload=json!({"type":kind});
            assert!(resource_value_from_detjson(&payload).is_err(),"{kind}: missing field must reject");
            payload[field]=JsonValue::Null;
            assert!(resource_value_from_detjson(&payload).is_err(),"{kind}: null field must reject");
            payload[field]=valid;
            assert!(resource_value_from_detjson(&payload).is_ok(),"{kind}: explicit empty/false must remain valid");
            let nested=json!({"type":"list","items":[{"type":kind}]});
            assert!(resource_value_from_detjson(&nested).is_err(),"{kind}: nested bad value must reject");
        }
    }

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
    fn unit_wire_number_admission_rejects_binary_float_and_keeps_textual_fixed64() {
        let half_ulp: JsonValue = serde_json::from_str("0.000000000116415321826934814453125")
            .expect("half ULP JSON number");
        let overflow: JsonValue = serde_json::from_str("2147483648")
            .expect("out-of-range JSON number");

        assert_eq!(json_to_fixed64(&half_ulp), None);
        assert_eq!(json_to_fixed64(&overflow), None);
        assert_eq!(
            json_to_fixed64(&json!("0.000000000349245965480804443359375")),
            Some(Fixed64::from_raw_i64(2))
        );
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
