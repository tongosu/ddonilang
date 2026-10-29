//! WASM 관찰 결과·세계 해시의 기존 도우미. 공개 API·수·오류 문구는 유지한다.
use super::*;

pub(super) fn current_state_hash(world: &DetNuri, view_prefixes: &[String]) -> String {
    let refs: Vec<&str> = view_prefixes.iter().map(String::as_str).collect();
    format!(
        "blake3:{}",
        world
            .world()
            .state_hash_excluding_resource_prefixes(&refs)
            .to_hex()
    )
}

pub(super) fn hash_json_string(value: &JsonValue) -> String {
    let canonical = value.to_string();
    let digest = blake3_hash(canonical.as_bytes());
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

pub(super) fn restore_hash_diags(
    payload: &JsonValue,
    actual_state_hash: &str,
    actual_view_hash: &str,
) -> Vec<JsonValue> {
    let mut diags = Vec::new();

    if let Some(expected_state_hash) = payload.get("state_hash").and_then(JsonValue::as_str) {
        if expected_state_hash != actual_state_hash {
            diags.push(json!({
                "code": "STATE_HASH_MISMATCH",
                "expected": expected_state_hash,
                "actual": actual_state_hash,
                "detail": "복원 후 state_hash 불일치",
            }));
        }
    }

    if let Some(expected_view_hash) = payload.get("view_hash").and_then(JsonValue::as_str) {
        if expected_view_hash != actual_view_hash {
            diags.push(json!({
                "code": "VIEW_HASH_MISMATCH",
                "expected": expected_view_hash,
                "actual": actual_view_hash,
                "detail": "복원 후 view_hash 불일치 (view_meta 변경 가능)",
            }));
        }
    }

    diags
}

pub(super) fn wasm_error(code: &str, detail: &str, hint: &str) -> JsValue {
    let payload = json!({
        "error": true,
        "code": code,
        "detail": detail,
        "hint": hint,
    });
    JsValue::from_str(&payload.to_string())
}

pub(super) fn fixed64_from_q31_32_parts(high_i32: i32, low_u32: u32) -> Fixed64 {
    let raw = (i64::from(high_i32) << 32) | i64::from(low_u32);
    Fixed64::from_raw_i64(raw)
}

pub(super) fn fixed64_q31_32_parts(value: Fixed64) -> JsonValue {
    let raw = value.raw_i64();
    json!({
        "high_i32": (raw >> 32) as i32,
        "low_u32": raw as u32,
    })
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
    }
    // 격자 전용 투영이 일반 그림판 출력을 가리지 않게 한다.
    // 두 투영 모두 같은 게시된 drawlist에서만 만들어진다.
    if let Some(space2d_json) = derive_space2d_from_bogae(world) {
        if let Ok(space2d) = serde_json::from_str::<JsonValue>(&space2d_json) {
            out.insert("space2d".to_string(), space2d);
            available_families.push("space2d");
        }
    }
    if let Some(graph_json) = runtime_surface::derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
            available_families.push("graph");
        }
    }
    if let Some(workbench) = build_workbench_authoritative_results(world, out.get("space2d")) {
        out.insert("workbench".to_string(), workbench);
    }

    let mut graph_hints = Vec::new();
    for (tag, id, label) in GRAPH_POINTS_TAGS.iter() {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(points) = points_from_resource_value(&value) else {
            continue;
        };
        if points.is_empty() {
            continue;
        }
        graph_hints.push(json!({
            "series_id": *id,
            "source": *tag,
            "y_label": *label,
            "overlay": true,
        }));
    }
    out.insert("graph_hints".to_string(), JsonValue::Array(graph_hints));
    append_view_stack_meta(&mut out, &available_families);

    JsonValue::Object(out)
}

pub(super) fn world_number(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> Option<f64> {
    world
        .get_resource_fixed64(tag)
        .map(fixed64_to_f64)
        .or_else(|| {
            world
                .get_resource_value(tag)
                .and_then(|value| resource_value_to_number(&value))
        })
        .filter(|value| value.is_finite())
}

pub(super) fn optional_world_number_json(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> JsonValue {
    world_number(world, tag).map_or(JsonValue::Null, |value| json!(value))
}

pub(super) fn workbench_result(schema: &str, family: &str, content: JsonValue) -> JsonValue {
    json!({
        "schema": schema,
        "family": family,
        "status": "available",
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "content": content,
    })
}

pub(super) fn space2d_dimensions(space2d: &JsonValue) -> Option<JsonValue> {
    let camera = space2d.get("camera")?.as_object()?;
    let x_min = camera.get("x_min")?.as_f64()?;
    let x_max = camera.get("x_max")?.as_f64()?;
    let y_min = camera.get("y_min")?.as_f64()?;
    let y_max = camera.get("y_max")?.as_f64()?;
    let columns = (x_max - x_min).abs();
    let rows = (y_max - y_min).abs();
    if !columns.is_finite() || !rows.is_finite() || columns <= 0.0 || rows <= 0.0 {
        return None;
    }
    Some(json!({ "columns": columns, "rows": rows }))
}

pub(super) fn build_workbench_authoritative_results(
    world: &ddonirang_core::platform::NuriWorld,
    space2d: Option<&JsonValue>,
) -> Option<JsonValue> {
    let mut results = Map::new();

    if let (Some(x), Some(y), Some(a), Some(b)) = (
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "a"),
        world_number(world, "b"),
    ) {
        results.insert(
            "line".to_string(),
            workbench_result(
                WORKBENCH_LINE_RESULT_SCHEMA,
                "line",
                json!({
                    "current": { "x": x, "y": y },
                    "equation": { "a": a, "b": b },
                }),
            ),
        );
    }

    if let (Some(theta), Some(omega), Some(length), Some(bob_x), Some(bob_y)) = (
        world_number(world, "theta"),
        world_number(world, "omega"),
        world_number(world, "L"),
        world_number(world, "bob_x"),
        world_number(world, "bob_y"),
    ) {
        if length > 0.0 {
            results.insert(
                "pendulum".to_string(),
                workbench_result(
                    WORKBENCH_PENDULUM_RESULT_SCHEMA,
                    "pendulum",
                    json!({
                        "current": {
                            "theta": theta,
                            "omega": omega,
                            "energy": optional_world_number_json(world, "energy"),
                            "kinetic_energy": optional_world_number_json(world, "kinetic_energy"),
                            "potential_energy": optional_world_number_json(world, "potential_energy"),
                            "length": length,
                            "physical_bob": { "x": bob_x, "y": bob_y },
                            "normalized_bob": { "x": bob_x / length, "y": -bob_y / length },
                        }
                    }),
                ),
            );
        }
    }

    if let (Some(t), Some(x), Some(y), Some(vx), Some(vy)) = (
        world_number(world, "t"),
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "vx"),
        world_number(world, "vy"),
    ) {
        results.insert(
            "projectile".to_string(),
            workbench_result(
                WORKBENCH_PROJECTILE_RESULT_SCHEMA,
                "projectile",
                json!({
                    "current": { "t": t, "x": x, "y": y, "vx": vx, "vy": vy },
                }),
            ),
        );
    }

    let grid_position = world
        .get_resource_value("위치")
        .and_then(|value| resource_value_to_point(&value));
    let grid_space2d = space2d.filter(|value| {
        value
            .get("drawlist")
            .and_then(JsonValue::as_array)
            .is_some_and(|items| !items.is_empty())
    });
    if let (Some((x, y)), Some(space2d)) = (grid_position, grid_space2d) {
        results.insert(
            "grid".to_string(),
            workbench_result(
                WORKBENCH_GRID_RESULT_SCHEMA,
                "grid",
                json!({
                    "position": { "x": x, "y": y },
                    "dimensions": space2d_dimensions(space2d),
                    "space2d": space2d.clone(),
                }),
            ),
        );
    }

    if results.is_empty() {
        return None;
    }
    Some(json!({
        "schema": WORKBENCH_RESULT_SET_SCHEMA,
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "results": results,
    }))
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

pub(super) fn collect_streams(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let sidecars = collect_stream_sidecars(world);
    for (tag, value) in world.resource_value_entries() {
        let meta = sidecars.get(&tag).copied();
        if !is_stream_resource(&tag, &value, meta.as_ref()) {
            continue;
        }
        let Some(stream) = extract_ring_buffer(&value, meta.as_ref()) else {
            continue;
        };
        out.insert(tag, stream);
    }
    JsonValue::Object(out)
}

