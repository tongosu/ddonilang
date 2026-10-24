//! 세계·입력·보개의 기존 표시 투영. 수 표시·원문·저장 규격은 변경하지 않는다.
use super::*;

pub fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
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
    defaults.insert("프레임수".to_string(), bogae_host_numeric_supply().frame_initial());
    defaults.insert(
        "__wasm_start_once".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("회전_x".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert("회전_y".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
}

pub fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let mut value_json_map = Map::new();
    out.insert("schema".to_string(), json!(RESOURCE_SNAPSHOT_SCHEMA_V2));
    out.insert(
        "unit_wire_schema".to_string(),
        json!(UNIT_VALUE_WIRE_SCHEMA_V2),
    );

    let mut json_map = Map::new();
    for (tag, json_text) in world.resource_json_entries() {
        value_json_map.insert(tag.clone(), resource_json_text_to_json(&json_text));
        json_map.insert(tag, JsonValue::String(json_text));
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
    let mut value_det_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag.clone(), JsonValue::String(value.canon_key()));
        value_json_map.insert(tag.clone(), resource_value_to_json(&value));
        value_det_map.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));
    out.insert("value_json".to_string(), JsonValue::Object(value_json_map));
    out.insert("value_det".to_string(), JsonValue::Object(value_det_map));

    JsonValue::Object(out)
}

pub(super) fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    out.insert("schema".to_string(), json!(RESOURCE_SNAPSHOT_SCHEMA_V2));
    out.insert(
        "unit_wire_schema".to_string(),
        json!(UNIT_VALUE_WIRE_SCHEMA_V2),
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

pub(crate) fn runtime_state_snapshot(
    world: &DetNuri,
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
) -> JsonValue {
    JsonValue::Object([
        ("schema".into(), "ddn.wasm_state_snapshot.v2".into()),
        ("resource_snapshot_schema".into(), RESOURCE_SNAPSHOT_SCHEMA_V2.into()),
        ("unit_wire_schema".into(), UNIT_VALUE_WIRE_SCHEMA_V2.into()),
        ("resources".into(), serialize_world_resources_snapshot(world.world())),
        ("param_overrides".into(), JsonValue::Object(Map::new())),
        ("public_observation_frames".into(), JsonValue::Array(public_observation_frames)),
        ("next_tick_id".into(), next_tick_id.into()),
    ].into_iter().collect())
}

pub fn collect_columns_and_row(
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
                items.push(resource_value_patch_json(tag, value));
            }
            PatchOp::SetSubjectWorld { value } => {
                items.push(json!({"op":"set_subject_world","subject_values_sha256":sha256_hex(&value.subject_values),
                    "subject_custody_sha256":sha256_hex(&value.subject_custody)}));
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

pub(crate) fn resource_value_patch_json(tag: &str, value: &ResourceValue) -> JsonValue {
    JsonValue::Object([
        ("op".into(), JsonValue::String("set_resource_value".into())),
        ("tag".into(), JsonValue::String(tag.into())),
        ("value".into(), JsonValue::String(value.canon_key())),
        ("value_json".into(), resource_value_to_json(value)),
    ].into_iter().collect())
}

pub(crate) fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    let mut out = Map::new();
    out.insert("input_encoding".into(), json!(crate::input_receipt::SNAPSHOT_ENCODING));
    out.insert("keys_pressed".to_string(), json!(input.keys_pressed));
    out.insert("last_key_name".to_string(), json!(input.last_key_name));
    out.insert("pointer_x_i32".to_string(), json!(input.pointer_x_i32));
    out.insert("pointer_y_i32".to_string(), json!(input.pointer_y_i32));
    out.insert("dt".to_string(), json!(crate::input_receipt::exact_dt_text(input.dt)));
    out.insert(
        "frame_source".to_string(),
        json!(input.frame_source.label()),
    );
    out.insert(
        "frame_source_code".to_string(),
        json!(input.frame_source.code_u8()),
    );
    out.insert("rng_seed".to_string(), json!(input.rng_seed));
    if let Some(base_seed) = rng_base_seed {
        out.insert("rng_base_seed".to_string(), json!(base_seed));
    }
    out.insert(
        "ai_injections".to_string(),
        JsonValue::Array(
            input
                .ai_injections
                .iter()
                .map(|packet| {
                    json!({
                        "agent_id": packet.agent_id,
                        "recv_seq": packet.recv_seq,
                        "accepted_madi": packet.accepted_madi,
                        "target_madi": packet.target_madi,
                        "source": packet.source.label(),
                        "source_code": packet.source.code_u8(),
                        "source_meta": input_source_json(packet.source),
                        "intent": seulgi_intent_json(&packet.intent),
                    })
                })
                .collect(),
        ),
    );
    out.insert(
        "net_events".to_string(),
        JsonValue::Array(
            input
                .net_events
                .iter()
                .map(|event| {
                    json!({
                        "sender": event.sender.as_str(),
                        "seq": event.seq,
                        "order_key": event.order_key.as_str(),
                        "payload_detjson": event.payload_detjson.as_str(),
                        "source": event.source.label(),
                        "source_code": event.source.code_u8(),
                        "source_meta": input_source_json(event.source),
                    })
                })
                .collect(),
        ),
    );
    JsonValue::Object(out)
}

pub(super) fn serialize_input_state(
    keys_pressed: u64,
    last_key_name: &str,
    pointer_x_i32: i32,
    pointer_y_i32: i32,
    dt: Fixed64,
    rng_seed: u64,
    rng_base_seed: u64,
) -> JsonValue {
    json!({
        "keys_pressed": keys_pressed,
        "last_key_name": last_key_name,
        "pointer_x_i32": pointer_x_i32,
        "pointer_y_i32": pointer_y_i32,
        "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
        "dt": crate::input_receipt::exact_dt_text(dt),
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": rng_seed,
        "rng_base_seed": rng_base_seed,
        "pending_ai_injection_count": 0,
    })
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

pub(super) fn resource_json_text_to_json(text: &str) -> JsonValue {
    serde_json::from_str::<JsonValue>(text).unwrap_or_else(|_| JsonValue::String(text.to_string()))
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

pub(crate) fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    // F5-GA[VIEW_JOBS] One root job produces one result; each private parent job consumes exactly its previously scheduled immutable child count, including empty containers.
    // Display projection has the same arbitrary saved-tree nesting as detjson.
    // Keep completed children owned until they are transferred into a parent.
    enum Job<'a> { Value(&'a ResourceValue), Array(usize), Object(Vec<String>) }
    let mut jobs = vec![Job::Value(value)];
    let mut built = Vec::<crate::unit_wire::SnapshotJsonOwner>::new();
    while let Some(job) = jobs.pop() {
        match job {
            Job::Array(count) => {
                let children = built.split_off(built.len() - count);
                built.push(crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Array(
                    children.into_iter().map(|child| child.into_parent_value()).collect())));
            }
            Job::Object(keys) => {
                let children = built.split_off(built.len() - keys.len());
                let mut object = Map::new();
                for (key, child) in keys.into_iter().zip(children) {
                    // Display keys can collide. Do not recursively drop the displaced value.
                    if let Some(previous) = object.insert(key, child.into_parent_value()) {
                        drop(crate::unit_wire::SnapshotJsonOwner::from_value(previous));
                    }
                }
                built.push(crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Object(object)));
            }
            Job::Value(value) => {
                if let Some(formula) = formula_from_resource_value(value) {
                    let (dialect, raw) = formula_summary_parts(&formula);
                    built.push(crate::unit_wire::SnapshotJsonOwner::from_value(json!({"dialect":dialect,"raw":raw})));
                    continue;
                }
                match value {
                    ResourceValue::List(items) => {
                        jobs.push(Job::Array(items.len()));
                        jobs.extend(items.iter().rev().map(Job::Value));
                    }
                    ResourceValue::Set(items) => {
                        jobs.push(Job::Array(items.len()));
                        jobs.extend(items.values().rev().map(Job::Value));
                    }
                    ResourceValue::Map(entries) => {
                        jobs.push(Job::Object(entries.values().map(|entry| resource_key_to_string(&entry.key)).collect()));
                        jobs.extend(entries.values().rev().map(|entry| Job::Value(&entry.value)));
                    }
                    _ => built.push(crate::unit_wire::SnapshotJsonOwner::from_value(resource_value_leaf_to_json(value))),
                }
            }
        }
    }
    built.pop().expect("display root").into_parent_value()
}

pub(super) fn resource_value_leaf_to_json(value: &ResourceValue) -> JsonValue {
    if let Some(formula) = formula_from_resource_value(value) {
        let (dialect, raw) = formula_summary_parts(&formula);
        return json!({
            "dialect": dialect,
            "raw": raw,
        });
    }
    match value {
        ResourceValue::Record { .. } | ResourceValue::Tuple { .. } | ResourceValue::RationalV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::IdentityReference(_) | ResourceValue::RegisteredAtom(_) | ResourceValue::Bytes(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::BasicTensorV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::NominalValueV1(_) | ResourceValue::SubjectHandleV1(_) | ResourceValue::SubjectPayloadV1(_) => crate::unit_wire::resource_value_to_detjson(value),
        ResourceValue::None => JsonValue::Null,
        ResourceValue::Bool(v) => JsonValue::Bool(*v),
        ResourceValue::Fixed64(v) => json!(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => json!(fixed64_to_f64(v.value)),
        ResourceValue::String(s) => JsonValue::String(s.clone()),
        ResourceValue::ResourceHandle(handle) => JsonValue::String(handle_to_string(*handle)),
        ResourceValue::List(_) | ResourceValue::Set(_) | ResourceValue::Map(_) => unreachable!("container is scheduled"),
    }
}

pub(super) fn build_view_meta(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    if let Some(width) = world.get_resource_fixed64(BOGAE_WIDTH_TAG) {
        out.insert("canvas_width".to_string(), json!(fixed64_to_f64(width)));
    }
    if let Some(height) = world.get_resource_fixed64(BOGAE_HEIGHT_TAG) {
        out.insert("canvas_height".to_string(), json!(fixed64_to_f64(height)));
    }

    if let Some(draw_list_value) = world.get_resource_value(BOGAE_DRAWLIST_TAG) {
        if let ResourceValue::List(items) = &draw_list_value {
            let draw_items: Vec<JsonValue> = items
                .iter()
                .filter_map(resource_value_to_draw_item)
                .collect();
            out.insert("draw_list".to_string(), JsonValue::Array(draw_items));
            out.insert(
                "draw_list_meta".to_string(),
                json!({
                    "deprecated": true,
                    "use_instead": "view_meta.space2d",
                }),
            );
        }
    }

    let mut available_families = Vec::new();
    if let Some(grid2d_json) = derive_grid2d_from_bogae(world) {
        if let Ok(grid2d) = serde_json::from_str::<JsonValue>(&grid2d_json) {
            out.insert("grid2d".to_string(), grid2d);
            available_families.push("grid2d");
        }
    } else if let Some(space2d_json) = derive_space2d_from_bogae(world) {
        if let Ok(space2d) = serde_json::from_str::<JsonValue>(&space2d_json) {
            out.insert("space2d".to_string(), space2d);
            available_families.push("space2d");
        }
    }
    if let Some(graph_json) = derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
            available_families.push("graph");
        }
    }
    append_view_stack_meta(&mut out, &available_families);
    JsonValue::Object(out)
}

pub(super) fn append_view_stack_meta(out: &mut Map<String, JsonValue>, available_families: &[&str]) {
    let Some((primary_family, secondary_families)) = available_families.split_first() else {
        return;
    };
    out.insert(
        "primary".to_string(),
        json!({
            "family": *primary_family,
            "role": "main",
        }),
    );
    let secondary = secondary_families
        .iter()
        .map(|family| {
            json!({
                "family": *family,
                "role": "secondary",
            })
        })
        .collect::<Vec<_>>();
    out.insert("secondary".to_string(), JsonValue::Array(secondary));
    out.insert("overlays".to_string(), JsonValue::Array(Vec::new()));
}

pub(crate) fn derive_graph_from_points(
    world: &ddonirang_core::platform::NuriWorld,
) -> Option<String> {
    let mut series = Vec::new();
    let mut single_series_labels = None;
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for tag in GRAPH_POINTS_TAGS {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(graph_series) = graph_series_from_resource_value(&value) else {
            continue;
        };
        if graph_series.points.is_empty() {
            continue;
        }
        for (x, y) in &graph_series.points {
            if x.is_finite() {
                x_min = x_min.min(*x);
                x_max = x_max.max(*x);
            }
            if y.is_finite() {
                y_min = y_min.min(*y);
                y_max = y_max.max(*y);
            }
        }
        let mut row = Map::new();
        row.insert(
            "id".to_string(),
            json!(graph_series.y_label.as_deref().unwrap_or(tag)),
        );
        row.insert("source_ref".to_string(), json!(*tag));
        row.insert(
            "points".to_string(),
            JsonValue::Array(
                graph_series
                    .points
                    .iter()
                    .map(|(x, y)| json!([x, y]))
                    .collect(),
            ),
        );
        if let (Some(label), Some(source_ref)) = (
            graph_series.y_label.as_ref(),
            graph_series.y_source_ref.as_ref(),
        ) {
            row.insert("label".to_string(), json!(label));
            row.insert("label_source_ref".to_string(), json!(source_ref));
        }
        single_series_labels = Some((
            graph_series.x_label,
            graph_series.x_source_ref,
            graph_series.y_label,
            graph_series.y_source_ref,
        ));
        series.push(JsonValue::Object(row));
    }
    if series.is_empty() {
        return None;
    }
    let mut graph = Map::new();
    graph.insert("schema".to_string(), json!(GRAPH_SCHEMA));
    graph.insert("series".to_string(), JsonValue::Array(series));
    if x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite() {
        let mut axis = Map::from_iter([
            ("x_min".to_string(), json!(x_min)),
            ("x_max".to_string(), json!(x_max)),
            ("y_min".to_string(), json!(y_min)),
            ("y_max".to_string(), json!(y_max)),
        ]);
        if graph
            .get("series")
            .and_then(JsonValue::as_array)
            .map(Vec::len)
            == Some(1)
        {
            if let Some((x_label, x_source_ref, y_label, y_source_ref)) = single_series_labels {
                if let (Some(label), Some(source_ref)) = (x_label, x_source_ref) {
                    axis.insert("x_label".to_string(), json!(label));
                    axis.insert("x_label_source_ref".to_string(), json!(source_ref));
                }
                if let (Some(label), Some(source_ref)) = (y_label, y_source_ref) {
                    axis.insert("y_label".to_string(), json!(label));
                    axis.insert("y_label_source_ref".to_string(), json!(source_ref));
                }
            }
        }
        graph.insert("axis".to_string(), JsonValue::Object(axis));
    }
    graph.insert(
        "meta".to_string(),
        json!({"graph_kind":"curve", "update":"replace", "source_kind":"runtime_resource"}),
    );
    Some(JsonValue::Object(graph).to_string())
}

#[derive(Default)]
pub(super) struct GraphSeriesResource {
    points: Vec<(f64, f64)>,
    x_label: Option<String>,
    x_source_ref: Option<String>,
    y_label: Option<String>,
    y_source_ref: Option<String>,
}

pub(super) fn graph_series_from_resource_value(value: &ResourceValue) -> Option<GraphSeriesResource> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut series = GraphSeriesResource::default();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            series.points.push(point);
        }
        let ResourceValue::Map(entries) = item else {
            continue;
        };
        if series.x_label.is_none() {
            series.x_label = resource_map_string(entries, "x_label");
        }
        if series.x_source_ref.is_none() {
            series.x_source_ref = resource_map_string(entries, "x_source_ref");
        }
        if series.y_label.is_none() {
            series.y_label = resource_map_string(entries, "y_label");
        }
        if series.y_source_ref.is_none() {
            series.y_source_ref = resource_map_string(entries, "y_source_ref");
        }
    }
    Some(series)
}

pub(super) fn resource_map_string(
    entries: &std::collections::BTreeMap<String, ddonirang_core::platform::ResourceMapEntry>,
    wanted: &str,
) -> Option<String> {
    entries.values().find_map(|entry| {
        if resource_key_to_string(&entry.key) != wanted {
            return None;
        }
        match &entry.value {
            ResourceValue::String(value) if !value.trim().is_empty() => Some(value.clone()),
            _ => None,
        }
    })
}

pub(super) fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                match resource_key_to_string(&entry.key).as_str() {
                    "x" => x = resource_value_to_number(&entry.value),
                    "y" => y = resource_value_to_number(&entry.value),
                    _ => {}
                }
            }
            Some((x?, y?))
        }
        ResourceValue::List(items) if items.len() >= 2 => Some((
            // F5-GA[VIEW_POINT] The same immutable list was proved to contain at least two entries by the match guard.
            resource_value_to_number(&items[0])?,
            resource_value_to_number(&items[1])?,
        )),
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
        JsonValue::String("seamgrim.space2d.v0".to_string()),
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
    }
    Some(JsonValue::Object(obj))
}

pub(super) fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

pub(super) fn map_bogae_kind(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("rect") {
        return "rect".to_string();
    }
    if lower.contains("text") {
        return "text".to_string();
    }
    if lower.contains("circle") {
        return "circle".to_string();
    }
    if lower.contains("polygon") {
        return "polygon".to_string();
    }
    if lower.contains("poly") {
        return "polyline".to_string();
    }
    if lower.contains("line") {
        return "line".to_string();
    }
    if lower.contains("point") {
        return "point".to_string();
    }
    if lower.contains("arrow") {
        return "arrow".to_string();
    }
    if lower.contains("sprite") {
        return "rect".to_string();
    }
    if lower.contains("사각") || lower.contains("네모") {
        return "rect".to_string();
    }
    if lower.contains("동그라미") || lower == "원" || lower.contains("원형") {
        return "circle".to_string();
    }
    if lower.contains("선분") || lower == "선" {
        return "line".to_string();
    }
    if lower == "점" || lower.contains("점형") {
        return "point".to_string();
    }
    if lower.contains("화살") || lower.contains("살표") {
        return "arrow".to_string();
    }
    if lower == "글" || lower.contains("글자") || lower.contains("문자") {
        return "text".to_string();
    }
    if lower.contains("다각") {
        return "polygon".to_string();
    }
    raw.to_string()
}

pub(super) fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

pub(super) fn hash_json_string(value: &JsonValue) -> String {
    // F5-GA[VIEW_JSON_VEC] The compact writer serializes only JsonValue/String keys and ASCII structure into a private Vec; it has no failing external sink or unsupported custom serializer.
    let mut canonical = Vec::new();
    crate::unit_wire::write_compact_json(&mut canonical, value).expect("JSON Vec writer");
    let digest = blake3_hash(&canonical);
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

pub(super) fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

pub(super) fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}

