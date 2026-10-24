//! WASM 자원·입력·보개 직렬화의 기존 도우미. 공개 API와 저장 형식은 그대로다.
use super::*;

pub(super) fn serialize_param_overrides(params: &BTreeMap<String, Value>) -> JsonValue {
    let mut out = Map::new();
    for (key, value) in params {
        let raw = match value {
            Value::Fixed64(v) => json!({ "raw_i64": v.raw_i64().to_string() }),
            Value::Bool(v) => JsonValue::Bool(*v),
            Value::String(v) => JsonValue::String(v.clone()),
            _ => continue,
        };
        out.insert(key.clone(), raw);
    }
    JsonValue::Object(out)
}

pub(super) fn parse_param_overrides(payload: &JsonValue) -> Result<BTreeMap<String, Value>, String> {
    crate::unit_wire::parse_param_overrides(payload)
}

pub(super) fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    out.insert(
        "schema".to_string(),
        JsonValue::String(RESOURCE_SNAPSHOT_SCHEMA_V2.to_string()),
    );
    out.insert(
        "unit_wire_schema".to_string(),
        JsonValue::String(UNIT_VALUE_WIRE_SCHEMA_V2.to_string()),
    );
    out.insert("next_entity".to_string(), json!(world.next_entity_cursor()));
    out.insert(
        "components".to_string(),
        JsonValue::Array(
            world
                .component_json_entries()
                .into_iter()
                .map(
                    |(entity, tag, value)| json!({"entity": entity.0, "tag": tag.0, "json": value}),
                )
                .collect(),
        ),
    );

    let mut json_map = Map::new();
    for (tag, value) in world.resource_json_entries() {
        json_map.insert(tag, JsonValue::String(value));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        fixed_map.insert(tag, json!({"raw_i64": value.raw_i64().to_string()}));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_det = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_det.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value_det".to_string(), JsonValue::Object(value_det));

    JsonValue::Object(out)
}

pub(super) fn restore_world_resources(
    world: &mut ddonirang_core::platform::NuriWorld,
    resources: &JsonValue,
) -> Result<(), String> {
    crate::unit_wire::restore_world_resources_snapshot(world,resources)
}

pub(super) fn parse_handle_string(text: &str) -> Result<u64, String> {
    let trimmed = text.trim();
    let without_handle = trimmed.strip_prefix("handle:").unwrap_or(trimmed);
    let hex = without_handle
        .strip_prefix("자원#")
        .unwrap_or(without_handle);
    u64::from_str_radix(hex, 16).map_err(|_| format!("handle 파싱 실패: {text}"))
}

pub(super) fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    crate::unit_wire::resource_value_to_detjson(value)
}

pub(super) fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    crate::unit_wire::resource_value_from_detjson(payload)
}

pub(super) fn json_to_u64(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_u64(),
        JsonValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

pub(super) fn json_to_i32(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
        JsonValue::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}

pub(super) fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let mut value_json_map = Map::new();

    let mut json_map = Map::new();
    for (tag, json) in world.resource_json_entries() {
        value_json_map.insert(tag.clone(), resource_json_text_to_json(&json));
        json_map.insert(tag, JsonValue::String(json));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        value_json_map.insert(tag.clone(), json!(fixed64_to_f64(value)));
        fixed_map.insert(tag, JsonValue::String(value.display_rounded_six()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag.clone(), JsonValue::String(value.canon_key()));
        value_json_map.insert(tag, resource_value_to_json(&value));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));
    out.insert("value_json".to_string(), JsonValue::Object(value_json_map));

    JsonValue::Object(out)
}

pub(super) fn resource_json_text_to_json(text: &str) -> JsonValue {
    serde_json::from_str::<JsonValue>(text).unwrap_or_else(|_| JsonValue::String(text.to_string()))
}

pub(super) fn input_source_json(source: InputSource) -> JsonValue {
    json!({
        "label": source.label(),
        "code": source.code_u8(),
    })
}

pub(super) fn seulgi_intent_json(intent: &SeulgiIntent) -> JsonValue {
    match intent {
        SeulgiIntent::None => json!({
            "kind": "none",
            "kind_code": intent.kind_u32(),
        }),
        SeulgiIntent::MoveTo { x, y } => json!({
            "kind": "move_to",
            "kind_code": intent.kind_u32(),
            "x": x.to_string(),
            "y": y.to_string(),
            "x_raw_i64": x.raw_i64(),
            "y_raw_i64": y.raw_i64(),
        }),
        SeulgiIntent::Attack { target_id } => json!({
            "kind": "attack",
            "kind_code": intent.kind_u32(),
            "target_id": target_id,
        }),
        SeulgiIntent::Say { text } => json!({
            "kind": "say",
            "kind_code": intent.kind_u32(),
            "text": text.as_str(),
        }),
    }
}

pub(super) fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    runtime_surface::serialize_input_snapshot(input, rng_base_seed)
}

pub(super) fn serialize_input_state(vm: &DdnWasmVm) -> JsonValue {
    json!({
        "alpha2_host_madi_hz": crate::input_receipt::alpha2_host_madi_hz(),
        "alpha_host_dt": crate::input_receipt::exact_dt_text(crate::input_receipt::alpha_host_dt()),
        "keys_pressed": vm.input_keys_pressed,
        "last_key_name": vm.input_last_key_name,
        "pointer_x_i32": vm.input_pointer_x_i32,
        "pointer_y_i32": vm.input_pointer_y_i32,
        "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
        "dt": crate::input_receipt::exact_dt_text(vm.input_dt),
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": vm.rng_seed,
        "rng_base_seed": vm.rng_seed,
        "pending_ai_injection_count": vm.pending_ai_injections.len(),
    })
}

pub(super) fn serialize_patch(patch: &Patch) -> JsonValue {
    let mut items = Vec::new();
    for op in &patch.ops {
        match op {
            PatchOp::SetResourceJson { tag, json } => {
                items.push(json!({ "op": "set_resource_json", "tag": tag, "value": json }));
            }
            PatchOp::SetResourceFixed64 { tag, value } => {
                items.push(
                    json!({ "op": "set_resource_fixed64", "tag": tag, "value": value.to_string() }),
                );
            }
            PatchOp::SetResourceHandle { tag, handle } => {
                items.push(json!({ "op": "set_resource_handle", "tag": tag, "value": handle_to_string(*handle) }));
            }
            PatchOp::SetResourceValue { tag, value } => {
                items.push(crate::runtime_surface::resource_value_patch_json(tag, value));
            }
            PatchOp::SetSubjectWorld { value } => {
                items.push(json!({"op":"set_subject_world",
                    "subject_values_sha256":runtime_surface::sha256_hex(&value.subject_values),
                    "subject_custody_sha256":runtime_surface::sha256_hex(&value.subject_custody)}));
            }
            PatchOp::DivAssignResourceFixed64 { tag, rhs, .. } => {
                items.push(json!({ "op": "div_assign_resource_fixed64", "tag": tag, "value": rhs.to_string() }));
            }
            PatchOp::SetComponentJson { entity, tag, json } => {
                items.push(json!({ "op": "set_component_json", "entity": entity.0, "tag": tag.0, "value": json }));
            }
            PatchOp::RemoveComponent { entity, tag } => {
                items.push(json!({ "op": "remove_component", "entity": entity.0, "tag": tag.0 }));
            }
            PatchOp::EmitSignal { signal, targets } => {
                items.push(
                    json!({ "op": "emit_signal", "signal": signal.name(), "targets": targets }),
                );
            }
            PatchOp::GuardViolation { entity, rule_id } => {
                items.push(
                    json!({ "op": "guard_violation", "entity": entity.0, "rule_id": rule_id }),
                );
            }
        }
    }
    JsonValue::Array(items)
}

pub(super) fn collect_columns_and_row(
    world: &ddonirang_core::platform::NuriWorld,
) -> (Vec<JsonValue>, Vec<JsonValue>) {
    let mut table: BTreeMap<String, (String, JsonValue)> = BTreeMap::new();

    for (tag, value) in world.resource_fixed64_entries() {
        table.insert(tag, ("num".to_string(), json!(fixed64_to_f64(value))));
    }

    for (tag, value) in world.resource_json_entries() {
        if let Some((dtype, scalar)) = json_text_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    for (tag, value) in world.resource_value_entries() {
        if let Some((dtype, scalar)) = resource_value_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    let mut columns = Vec::with_capacity(table.len());
    let mut row = Vec::with_capacity(table.len());
    for (key, (dtype, value)) in table {
        columns.push(json!({
            "key": key,
            "dtype": dtype,
            "role": "state",
        }));
        row.push(value);
    }
    (columns, row)
}

pub(super) fn json_text_to_scalar(text: &str) -> Option<(String, JsonValue)> {
    let trimmed = text.trim();
    if trimmed == "참" {
        return Some(("bool".to_string(), JsonValue::Bool(true)));
    }
    if trimmed == "거짓" {
        return Some(("bool".to_string(), JsonValue::Bool(false)));
    }
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
        return match parsed {
            JsonValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(v))),
            JsonValue::Number(v) => Some(("num".to_string(), JsonValue::Number(v))),
            JsonValue::String(v) => Some(("str".to_string(), JsonValue::String(v))),
            _ => None,
        };
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.is_finite() {
            return Some(("num".to_string(), json!(number)));
        }
    }
    Some(("str".to_string(), JsonValue::String(text.to_string())))
}

pub(super) fn resource_value_to_scalar(value: &ResourceValue) -> Option<(String, JsonValue)> {
    match value {
        ResourceValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(*v))),
        ResourceValue::Fixed64(v) => Some(("num".to_string(), json!(fixed64_to_f64(*v)))),
        ResourceValue::Unit(v) => Some(("num".to_string(), json!(fixed64_to_f64(v.value)))),
        ResourceValue::String(v) => Some(("str".to_string(), JsonValue::String(v.clone()))),
        _ => None,
    }
}

pub(super) fn js_scalar_to_runtime_value(value: &JsValue) -> Option<Value> {
    if value.as_f64().is_some() {
        return None;
    }
    if let Some(v) = value.as_bool() {
        return Some(Value::Bool(v));
    }
    value.as_string().map(Value::String)
}

pub(super) fn apply_param_value(
    world: &mut ddonirang_core::platform::NuriWorld,
    key: &str,
    value: &Value,
) -> Result<(), String> {
    world.validate_typed_resource_lanes().map_err(str::to_string)?;
    let fixed=matches!(value,Value::Fixed64(_));
    if world.get_resource_handle(key).is_some()
        || (fixed && world.get_resource_value(key).is_some())
        || (!fixed && world.get_resource_fixed64(key).is_some()) {
        return Err("같은 자원 이름이 여러 저장 통로에 있습니다".into());
    }
    match value {
        Value::Fixed64(v) => {
            world.set_resource_fixed64(key.to_string(), *v);
            Ok(())
        }
        Value::Bool(v) => {
            world.set_resource_value(key.to_string(), ResourceValue::Bool(*v));
            Ok(())
        }
        Value::String(v) => {
            world.set_resource_value(key.to_string(), ResourceValue::String(v.clone()));
            Ok(())
        }
        _ => Err("set_param: 수/참거짓/글 스칼라만 허용됩니다".to_string()),
    }
}

pub(super) fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

pub(super) fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
    defaults.insert(BOGAE_DRAWLIST_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(BOGAE_SHOW_LINES_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(
        BOGAE_WIDTH_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        BOGAE_HEIGHT_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("프레임수".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert(
        "__wasm_start_once".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("회전_x".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert("회전_y".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
}

pub(super) fn derive_space2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = &list else {
        return None;
    };
    let mut draw_items = Vec::new();
    for item in items {
        if let Some(obj) = resource_value_to_draw_item(item) {
            draw_items.push(obj);
        }
    }
    if draw_items.is_empty() {
        return None;
    }
    let mut space2d = Map::new();
    space2d.insert(
        "schema".to_string(),
        JsonValue::String(SPACE2D_SCHEMA.to_string()),
    );
    space2d.insert("drawlist".to_string(), JsonValue::Array(draw_items));
    if let (Some(w), Some(h)) = (
        world.get_resource_fixed64(BOGAE_WIDTH_TAG),
        world.get_resource_fixed64(BOGAE_HEIGHT_TAG),
    ) {
        let w = fixed64_to_f64(w);
        let h = fixed64_to_f64(h);
        if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
            space2d.insert(
                "camera".to_string(),
                json!({
                    "x_min": 0.0,
                    "x_max": w,
                    "y_min": 0.0,
                    "y_max": h,
                }),
            );
        }
    }
    Some(JsonValue::Object(space2d).to_string())
}

pub(super) fn derive_grid2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = &list else {
        return None;
    };
    let draw_items: Vec<JsonValue> = items
        .iter()
        .filter_map(resource_value_to_draw_item)
        .collect();
    if draw_items.is_empty()
        || !draw_items.iter().all(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .is_some_and(|id| id.starts_with("격자게임셀_") || id.starts_with("cell."))
        })
    {
        return None;
    }
    Some(
        json!({
            "schema": "seamgrim.grid2d.v0",
            "drawlist": draw_items,
            "source": "std_grid_game_bogae_drawlist"
        })
        .to_string(),
    )
}

pub(super) fn points_from_resource_value(value: &ResourceValue) -> Option<Vec<(f64, f64)>> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            out.push(point);
        }
    }
    Some(out)
}

pub(super) fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                if key == "x" {
                    x = resource_value_to_number(&entry.value);
                } else if key == "y" {
                    y = resource_value_to_number(&entry.value);
                }
            }
            match (x, y) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            }
        }
        ResourceValue::List(items) => {
            if items.len() < 2 {
                return None;
            }
            Some((
                resource_value_to_number(&items[0])?,
                resource_value_to_number(&items[1])?,
            ))
        }
        _ => None,
    }
}

pub(super) fn resource_value_to_number(value: &ResourceValue) -> Option<f64> {
    match value {
        ResourceValue::Fixed64(v) => Some(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => Some(fixed64_to_f64(v.value)),
        ResourceValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        ResourceValue::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

pub(super) fn resource_value_to_draw_item(value: &ResourceValue) -> Option<JsonValue> {
    let ResourceValue::Map(entries) = value else {
        return None;
    };
    let mut obj = Map::new();
    let mut kind_raw: Option<String> = None;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        let json_value = resource_value_to_json(&entry.value);
        match key.as_str() {
            "결" | "kind" | "shape" | "도형" | "형태" => {
                kind_raw = Some(resource_value_to_string(&entry.value));
            }
            "채움색" | "채움" => {
                obj.insert("fill".to_string(), json_value);
            }
            "색" | "선색" => {
                obj.insert("color".to_string(), json_value);
            }
            "굵기" => {
                obj.insert("width".to_string(), json_value);
            }
            "크기" => {
                obj.insert("size".to_string(), json_value);
            }
            "글" | "내용" => {
                obj.insert("text".to_string(), json_value);
            }
            "층" | "레이어" | "layer" | "layerIndex" | "z" | "z_index" => {
                obj.insert("layer_index".to_string(), json_value);
            }
            "그룹" | "묶음" | "group" | "groupId" => {
                obj.insert("group_id".to_string(), json_value);
            }
            _ => {
                obj.insert(key, json_value);
            }
        }
    }
    if let Some(raw) = kind_raw {
        obj.insert("kind".to_string(), JsonValue::String(map_bogae_kind(&raw)));
        // 기존 격자 생산자의 정확한 Rect 식별자만 표준 꼭짓점으로 투영한다.
        // 세계의 x/y/w/h 값은 변경하지 않고, 잘못된 값은 그대로 거부된다.
        if raw == "#보개/2D.Rect" {
            if let (Some(x), Some(y), Some(w), Some(h)) = (
                obj.get("x").and_then(JsonValue::as_f64),
                obj.get("y").and_then(JsonValue::as_f64),
                obj.get("w").and_then(JsonValue::as_f64),
                obj.get("h").and_then(JsonValue::as_f64),
            ) {
                if [x, y, w, h, x + w, y + h].iter().all(|n| n.is_finite()) {
                    obj.insert("x1".to_string(), json!(x));
                    obj.insert("y1".to_string(), json!(y));
                    obj.insert("x2".to_string(), json!(x + w));
                    obj.insert("y2".to_string(), json!(y + h));
                }
            }
        }
    }
    Some(JsonValue::Object(obj))
}

pub(super) fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

pub(super) fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

pub(super) fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    crate::runtime_surface::resource_value_to_json(value)
}

pub(super) fn map_bogae_kind(raw: &str) -> String {
    // A draw kind is an existing typed projection identity.  Do not infer it
    // from a display label, case-folded spelling, or a substring: unknown
    // values are deliberately preserved for the projection boundary to reject.
    match raw {
        "rect" | "사각형" | "#보개/2D.Rect" => "rect".to_string(),
        "text" | "글" => "text".to_string(),
        "circle" | "원" => "circle".to_string(),
        "polygon" | "다각형" => "polygon".to_string(),
        "polyline" | "선들" => "polyline".to_string(),
        "line" | "선" => "line".to_string(),
        "point" | "점" => "point".to_string(),
        "arrow" | "화살표" => "arrow".to_string(),
        _ => raw.to_string(),
    }
}

pub(super) fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}
