#[cfg(not(target_arch = "wasm32"))]
use crate::artifact_output::write_text_artifact_atomic;
use crate::ddn_runtime::PublicObservationEvent;
use serde_json::{json, Map, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

pub const PUBLIC_OBSERVATION_TRACE_SCHEMA: &str = "ddn.public_observation_trace.v1";
pub const PUBLIC_OBSERVATION_TABLE_PROJECTION_SCHEMA: &str =
    "ddn.public_observation_table_projection.v2";
pub const DEBUG_PLAN_SCHEMA: &str = "seamgrim.ddn_debug_plan.candidate.v1";
pub const DEBUG_DIFF_SCHEMA: &str = "seamgrim.ddn_committed_diff.candidate.v1";
pub const DEBUG_EVIDENCE_SCHEMA: &str = "seamgrim.ddn_debug_evidence.candidate.v1";

fn sha256_json(value: &JsonValue) -> String {
    let bytes = serde_json::to_vec(&canonical_json(value)).unwrap_or_default();
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn canonical_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let sorted: BTreeMap<String, JsonValue> = map
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json(value)))
                .collect();
            JsonValue::Object(sorted.into_iter().collect())
        }
        JsonValue::Array(items) => JsonValue::Array(items.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn text(value: Option<&JsonValue>) -> String {
    value
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn require_text(obj: &Map<String, JsonValue>, key: &str) -> Result<String, String> {
    let value = text(obj.get(key));
    if value.is_empty() {
        Err(format!("E_DEBUG_REQUIRED_IDENTITY: {key}가 비어 있습니다"))
    } else {
        Ok(value)
    }
}

fn canonical_dataset_target(value: &JsonValue) -> Result<JsonValue, String> {
    let obj = value.as_object().ok_or_else(|| {
        "E_PUBLIC_OBSERVATION_DATASET_TARGET_BAD_TYPE: dataset_target은 object여야 합니다"
            .to_string()
    })?;
    if obj.len() != 3
        || !obj.contains_key("dataset_content_sha256")
        || !obj.contains_key("row_source_ordinal")
        || !obj.contains_key("column_name")
    {
        return Err(
            "E_PUBLIC_OBSERVATION_DATASET_TARGET_FIELDS: dataset target은 content hash, row ordinal, column name만 가져야 합니다"
                .to_string(),
        );
    }
    let dataset_content_sha256 = require_text(obj, "dataset_content_sha256")?;
    if dataset_content_sha256.len() != 64
        || !dataset_content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "E_PUBLIC_OBSERVATION_DATASET_TARGET_HASH: dataset content hash가 올바르지 않습니다"
                .to_string(),
        );
    }
    let row_source_ordinal = obj
        .get("row_source_ordinal")
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_DATASET_TARGET_ROW: row_source_ordinal은 u64여야 합니다"
                .to_string()
        })?;
    let column_name = require_text(obj, "column_name")?;
    Ok(canonical_json(&json!({
        "dataset_content_sha256": dataset_content_sha256,
        "row_source_ordinal": row_source_ordinal,
        "column_name": column_name,
    })))
}

fn canonical_observation(mut row: JsonValue) -> Result<JsonValue, String> {
    let obj = row
        .as_object_mut()
        .ok_or_else(|| "E_DEBUG_OBSERVATION_BAD_ROW: 공개 관찰은 object여야 합니다".to_string())?;
    if obj.contains_key("observation_id") || obj.contains_key("declaration_index") {
        return Err(
            "E_BOIM_SYNTHETIC_ID_REMOVED: 공개 관찰 정체성은 DDN 이름과 source_ref입니다"
                .to_string(),
        );
    }
    for key in ["name", "dtype", "source_ref", "value_canon"] {
        require_text(obj, key)?;
    }
    if !obj
        .get("starts_observation_event")
        .is_some_and(JsonValue::is_boolean)
    {
        return Err(
            "E_DEBUG_OBSERVATION_EVENT_BOUNDARY_MISSING: 공개 관찰문 시작 경계가 필요합니다"
                .to_string(),
        );
    }
    obj.entry("unit".to_string()).or_insert_with(|| json!(""));
    obj.entry("format".to_string())
        .or_insert_with(|| json!("ddn-canonical"));
    obj.entry("value_text".to_string())
        .or_insert_with(|| json!(""));
    obj.insert("status".to_string(), json!("present"));
    if let Some(dataset_target) = obj.get("dataset_target").cloned() {
        obj.insert(
            "dataset_target".to_string(),
            canonical_dataset_target(&dataset_target)?,
        );
    }
    Ok(canonical_json(&row))
}

pub fn canonical_public_observation_events(
    rows: impl IntoIterator<Item = JsonValue>,
) -> Result<Vec<JsonValue>, String> {
    let mut out = Vec::new();
    for row in rows {
        let canonical = canonical_observation(row)?;
        out.push(canonical);
    }
    Ok(out)
}

pub fn public_observation_events_json(
    events: &[PublicObservationEvent],
) -> Result<Vec<JsonValue>, String> {
    canonical_public_observation_events(events.iter().map(|event| {
        let mut row = json!({
            "committed_madi": event.committed_madi,
            "starts_observation_event": event.starts_observation_event,
            "name": event.name,
            "source_ref": event.source_ref,
            "source_start": event.source_start,
            "source_end": event.source_end,
            "dtype": event.dtype,
            "unit": event.unit,
            "format": event.format,
            "value_canon": event.value_canon,
            "value_text": event.value_text,
        });
        if let Some(target) = &event.dataset_target {
            row.as_object_mut()
                .expect("public observation event")
                .insert(
                    "dataset_target".to_string(),
                    json!({
                        "dataset_content_sha256": target.dataset_content_sha256,
                        "row_source_ordinal": target.row_source_ordinal,
                        "column_name": target.column_name,
                    }),
                );
        }
        row
    }))
}

fn observation_identity_key(row: &JsonValue) -> Result<String, String> {
    let obj = row
        .as_object()
        .ok_or_else(|| "E_DEBUG_OBSERVATION_BAD_ROW: 공개 관찰은 object여야 합니다".to_string())?;
    Ok(format!(
        "{}\0{}",
        require_text(obj, "name")?,
        require_text(obj, "source_ref")?
    ))
}

fn observation_identity(row: &JsonValue) -> JsonValue {
    json!({
        "name": text(row.get("name")),
        "source_ref": text(row.get("source_ref")),
    })
}

fn canonical_sparse_status(mut row: JsonValue) -> Result<JsonValue, String> {
    let obj = row
        .as_object_mut()
        .ok_or_else(|| "E_DEBUG_STATUS_BAD_ROW: 희소 표본 상태는 object여야 합니다".to_string())?;
    if obj.contains_key("observation_id") || obj.contains_key("declaration_index") {
        return Err(
            "E_BOIM_SYNTHETIC_ID_REMOVED: 희소 표본 정체성은 DDN 이름과 source_ref입니다"
                .to_string(),
        );
    }
    for key in ["name", "source_ref", "status"] {
        require_text(obj, key)?;
    }
    if !obj
        .get("starts_observation_event")
        .is_some_and(JsonValue::is_boolean)
    {
        return Err(
            "E_DEBUG_STATUS_EVENT_BOUNDARY_MISSING: 희소 관찰문 시작 경계가 필요합니다".to_string(),
        );
    }
    let status = text(obj.get("status"));
    match status.as_str() {
        "gap" | "pruned" => {
            if obj.get("reason").is_some() {
                return Err(format!(
                    "E_DEBUG_STATUS_REASON_FORBIDDEN: {status}에는 missing 이유를 붙일 수 없습니다"
                ));
            }
        }
        "missing" => {
            let reason = require_text(obj, "reason")?;
            if !matches!(reason.as_str(), "#계산실패" | "#원천없음" | "#기록손상") {
                return Err(format!("E_DEBUG_MISSING_REASON_UNSUPPORTED: {reason}"));
            }
        }
        "present" => {
            return Err(
                "E_DEBUG_PRESENT_EVENT_REQUIRED: present는 Rust 공개 관찰 사건으로만 기록합니다"
                    .to_string(),
            )
        }
        _ => return Err(format!("E_DEBUG_STATUS_UNSUPPORTED: {status}")),
    }
    obj.entry("dtype".to_string())
        .or_insert_with(|| json!("unknown"));
    obj.entry("unit".to_string()).or_insert_with(|| json!(""));
    obj.entry("format".to_string())
        .or_insert_with(|| json!("ddn-canonical"));
    Ok(canonical_json(&row))
}

fn canonical_frame(
    frame: &JsonValue,
    inherited: &Map<String, JsonValue>,
) -> Result<JsonValue, String> {
    let obj = frame
        .as_object()
        .ok_or_else(|| "E_DEBUG_FRAME_BAD_ROW: committed frame은 object여야 합니다".to_string())?;
    let madi = obj
        .get("committed_madi")
        .or_else(|| obj.get("madi"))
        .or_else(|| obj.get("tick_id"))
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| "E_DEBUG_FRAME_MADI_MISSING: committed_madi가 필요합니다".to_string())?;
    let state_hash = require_text(obj, "state_hash")?;
    let observations = match obj.get("public_observations") {
        Some(JsonValue::Array(rows)) => canonical_public_observation_events(rows.iter().cloned())?,
        Some(_) => {
            return Err(
                "E_DEBUG_OBSERVATIONS_BAD_TYPE: public_observations는 array여야 합니다".to_string(),
            )
        }
        None => Vec::new(),
    };
    let sparse_statuses = match obj.get("public_observation_statuses") {
        Some(JsonValue::Array(rows)) => rows
            .iter()
            .cloned()
            .map(canonical_sparse_status)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => {
            return Err(
                "E_DEBUG_STATUSES_BAD_TYPE: public_observation_statuses는 array여야 합니다"
                    .to_string(),
            )
        }
        None => Vec::new(),
    };
    let input = canonical_json(
        obj.get("input_snapshot")
            .or_else(|| obj.get("input"))
            .unwrap_or(&JsonValue::Null),
    );
    let mut result = json!({
        "committed_madi": madi,
        "input_snapshot": input,
        "public_observations": observations,
        "public_observation_statuses": sparse_statuses,
        "state_hash": state_hash,
        "checkpoint": obj.get("checkpoint").cloned().unwrap_or(JsonValue::Null),
        "retention": text(obj.get("retention")).to_lowercase(),
    });
    let out = result.as_object_mut().expect("frame object");
    for key in [
        "run_id",
        "source_identity",
        "revision_identity",
        "lock_identity",
    ] {
        let value = text(obj.get(key)).to_string();
        let inherited_value = text(inherited.get(key));
        out.insert(
            key.to_string(),
            json!(if value.is_empty() {
                inherited_value
            } else {
                value
            }),
        );
    }
    Ok(canonical_json(&result))
}

pub fn build_public_observation_trace(input: &JsonValue) -> Result<JsonValue, String> {
    let root = input
        .as_object()
        .ok_or_else(|| "E_DEBUG_RUN_BAD_INPUT: run record input은 object여야 합니다".to_string())?;
    let run_id = require_text(root, "run_id")?;
    let source_identity = require_text(root, "source_identity")?;
    let revision_identity = require_text(root, "revision_identity")?;
    let frames = root
        .get("frames")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "E_DEBUG_RUN_FRAMES_MISSING: frames가 필요합니다".to_string())?;
    let mut committed = frames
        .iter()
        .map(|frame| canonical_frame(frame, root))
        .collect::<Result<Vec<_>, _>>()?;
    committed.sort_by_key(|frame| {
        frame
            .get("committed_madi")
            .and_then(JsonValue::as_u64)
            .unwrap_or(u64::MAX)
    });
    let mut seen = BTreeSet::new();
    for frame in &committed {
        let madi = frame
            .get("committed_madi")
            .and_then(JsonValue::as_u64)
            .unwrap_or(u64::MAX);
        if !seen.insert(madi) {
            return Err(format!("E_DEBUG_RUN_MADI_DUPLICATE: committed madi {madi}"));
        }
        if text(frame.get("run_id")) != run_id
            || text(frame.get("source_identity")) != source_identity
            || text(frame.get("revision_identity")) != revision_identity
        {
            return Err(
                "E_DEBUG_RUN_IDENTITY_MIXED: 다른 run/source/revision frame을 섞을 수 없습니다"
                    .to_string(),
            );
        }
    }
    let run_status = require_text(root, "run_status")?;
    if !matches!(run_status.as_str(), "running" | "completed" | "failed") {
        return Err(format!("E_DEBUG_RUN_STATUS_UNSUPPORTED: {run_status}"));
    }
    let run_failure = if run_status == "failed" {
        let failure = root
            .get("run_failure")
            .and_then(JsonValue::as_object)
            .ok_or_else(|| {
                "E_DEBUG_RUN_FAILURE_MISSING: failed 실행에는 run_failure가 필요합니다".to_string()
            })?;
        json!({
            "code": require_text(failure, "code")?,
            "message": require_text(failure, "message")?,
        })
    } else {
        if root.get("run_failure").is_some() {
            return Err(
                "E_DEBUG_RUN_FAILURE_FORBIDDEN: 실패하지 않은 실행에는 run_failure를 붙일 수 없습니다"
                    .to_string(),
            );
        }
        JsonValue::Null
    };

    let mut columns = Vec::new();
    let mut column_index = BTreeMap::new();
    let mut samples = Vec::with_capacity(committed.len());
    for frame in &committed {
        let mut cells = Vec::new();
        let mut sample_identities = BTreeSet::new();
        let present = frame
            .get("public_observations")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();
        let sparse = frame
            .get("public_observation_statuses")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();
        let frame_has_no_events = present.is_empty() && sparse.is_empty();
        for cell in present.into_iter().chain(sparse) {
            let starts_observation_event = cell
                .get("starts_observation_event")
                .and_then(JsonValue::as_bool)
                .ok_or_else(|| {
                    "E_DEBUG_OBSERVATION_EVENT_BOUNDARY_MISSING: starts_observation_event가 필요합니다"
                        .to_string()
                })?;
            if starts_observation_event && !cells.is_empty() {
                samples.push(json!({
                    "committed_madi": frame.get("committed_madi").cloned().unwrap_or(JsonValue::Null),
                    "state_hash": text(frame.get("state_hash")),
                    "cells": cells,
                }));
                cells = Vec::new();
                sample_identities.clear();
            } else if !starts_observation_event && cells.is_empty() {
                return Err(
                    "E_DEBUG_OBSERVATION_EVENT_BOUNDARY_INVALID: 관찰 사건의 첫 필드가 경계를 열어야 합니다"
                        .to_string(),
                );
            }
            let identity_key = observation_identity_key(&cell)?;
            if sample_identities.contains(&identity_key) {
                return Err(format!(
                    "E_DEBUG_OBSERVATION_EVENT_IDENTITY_DUPLICATE: {} @ {}",
                    text(cell.get("name")),
                    text(cell.get("source_ref"))
                ));
            }
            let column = json!({
                "identity": observation_identity(&cell),
                "dtype": text(cell.get("dtype")),
                "unit": text(cell.get("unit")),
                "format": text(cell.get("format")),
            });
            if let Some(index) = column_index.get(&identity_key).copied() {
                if columns.get(index) != Some(&column) {
                    return Err(format!(
                        "E_DEBUG_OBSERVATION_BINDING_CHANGED: {} @ {}",
                        text(cell.get("name")),
                        text(cell.get("source_ref"))
                    ));
                }
            } else {
                column_index.insert(identity_key.clone(), columns.len());
                columns.push(column);
            }
            let mut sample = json!({
                "identity": observation_identity(&cell),
                "status": text(cell.get("status")),
            });
            if text(cell.get("status")) == "present" {
                let out = sample.as_object_mut().expect("sample object");
                out.insert(
                    "value_canon".to_string(),
                    cell.get("value_canon").cloned().unwrap_or(JsonValue::Null),
                );
                out.insert(
                    "value_text".to_string(),
                    cell.get("value_text").cloned().unwrap_or(JsonValue::Null),
                );
                if let Some(dataset_target) = cell.get("dataset_target") {
                    out.insert(
                        "dataset_target".to_string(),
                        canonical_dataset_target(dataset_target)?,
                    );
                }
            } else if text(cell.get("status")) == "missing" {
                sample.as_object_mut().expect("sample object").insert(
                    "reason".to_string(),
                    cell.get("reason").cloned().unwrap_or(JsonValue::Null),
                );
            }
            cells.push(sample);
            sample_identities.insert(identity_key);
        }
        if !cells.is_empty() || frame_has_no_events {
            samples.push(json!({
                "committed_madi": frame.get("committed_madi").cloned().unwrap_or(JsonValue::Null),
                "state_hash": text(frame.get("state_hash")),
                "cells": cells,
            }));
        }
    }

    let trace = json!({
        "schema": PUBLIC_OBSERVATION_TRACE_SCHEMA,
        "project_id": text(root.get("project_id")),
        "revision_identity": revision_identity,
        "source_identity": source_identity,
        "lock_identity": text(root.get("lock_identity")),
        "run_id": run_id,
        "parent_run_id": text(root.get("parent_run_id")),
        "branch_madi": root.get("branch_madi").cloned().unwrap_or(JsonValue::Null),
        "branch_state_hash": text(root.get("branch_state_hash")),
        "run_status": run_status,
        "run_failure": run_failure,
        "retention": root.get("retention").cloned().unwrap_or_else(|| json!({"limit": 2000})),
        "columns": columns,
        "samples": samples,
    });
    let trace = canonical_json(&trace);
    let trace_hash = public_observation_trace_hash(&trace);
    Ok(json!({
        "trace": trace,
        "trace_hash": trace_hash,
    }))
}

pub fn public_observation_trace_hash(trace: &JsonValue) -> String {
    let mut projection = trace.clone();
    if let Some(samples) = projection
        .get_mut("samples")
        .and_then(JsonValue::as_array_mut)
    {
        for sample in samples {
            if let Some(obj) = sample.as_object_mut() {
                obj.remove("state_hash");
            }
        }
    }
    sha256_json(&projection)
}

pub fn public_observation_table_projection_hash(projection: &JsonValue) -> String {
    let mut hash_input = projection.clone();
    if let Some(object) = hash_input.as_object_mut() {
        object.remove("projection_hash");
    }
    sha256_json(&hash_input)
}

fn seal_public_observation_table_projection(projection: JsonValue) -> JsonValue {
    let mut projection = canonical_json(&projection);
    let projection_hash = public_observation_table_projection_hash(&projection);
    projection
        .as_object_mut()
        .expect("public observation table projection")
        .insert("projection_hash".to_string(), json!(projection_hash));
    canonical_json(&projection)
}

/// Appends one or more newly committed frames to an already sealed public
/// observation surface. The result is byte-equivalent to rebuilding the trace
/// and table projection from the complete frame history, while avoiding a
/// second semantic projection pass over every older frame.
pub fn append_public_observation_surface(
    current_trace: &JsonValue,
    current_projection: &JsonValue,
    delta_input: &JsonValue,
) -> Result<(JsonValue, String, JsonValue), String> {
    let delta_bundle = build_public_observation_trace(delta_input)?;
    let delta_trace = delta_bundle
        .get("trace")
        .ok_or_else(|| "E_PUBLIC_OBSERVATION_DELTA_TRACE_MISSING: trace가 없습니다".to_string())?;
    let delta_run_id = require_text(
        delta_trace.as_object().ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_DELTA_TRACE_BAD_INPUT: trace는 object여야 합니다".to_string()
        })?,
        "run_id",
    )?;
    let delta_projection = build_public_observation_table_projection(delta_trace, &delta_run_id)?;

    let mut trace = current_trace.clone();
    let trace_root = trace.as_object_mut().ok_or_else(|| {
        "E_PUBLIC_OBSERVATION_APPEND_TRACE_BAD_INPUT: current trace는 object여야 합니다".to_string()
    })?;
    let delta_trace_root = delta_trace.as_object().expect("validated delta trace");
    if text(trace_root.get("schema")) != PUBLIC_OBSERVATION_TRACE_SCHEMA {
        return Err(format!(
            "E_PUBLIC_OBSERVATION_APPEND_TRACE_SCHEMA: {}",
            text(trace_root.get("schema"))
        ));
    }
    for key in [
        "project_id",
        "source_identity",
        "revision_identity",
        "lock_identity",
        "parent_run_id",
        "branch_state_hash",
    ] {
        if text(trace_root.get(key)) != text(delta_trace_root.get(key)) {
            return Err(format!(
                "E_PUBLIC_OBSERVATION_APPEND_IDENTITY_MISMATCH: {key}"
            ));
        }
    }
    if trace_root.get("branch_madi") != delta_trace_root.get("branch_madi") {
        return Err("E_PUBLIC_OBSERVATION_APPEND_IDENTITY_MISMATCH: branch_madi".to_string());
    }

    let delta_columns = delta_trace_root
        .get("columns")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_COLUMNS_MISSING: columns가 필요합니다".to_string()
        })?;
    let columns = trace_root
        .get_mut("columns")
        .and_then(JsonValue::as_array_mut)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_COLUMNS_MISSING: current columns가 필요합니다".to_string()
        })?;
    let mut column_index = BTreeMap::new();
    for (index, column) in columns.iter().enumerate() {
        column_index.insert(trace_identity_key(column)?.0, index);
    }
    for column in delta_columns {
        let key = trace_identity_key(column)?.0;
        if let Some(index) = column_index.get(&key).copied() {
            if columns.get(index) != Some(column) {
                return Err(format!(
                    "E_DEBUG_OBSERVATION_BINDING_CHANGED: {}",
                    key.replace('\0', " @ ")
                ));
            }
        } else {
            column_index.insert(key, columns.len());
            columns.push(column.clone());
        }
    }
    let delta_samples = delta_trace_root
        .get("samples")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_SAMPLES_MISSING: samples가 필요합니다".to_string()
        })?;
    trace_root
        .get_mut("samples")
        .and_then(JsonValue::as_array_mut)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_SAMPLES_MISSING: current samples가 필요합니다".to_string()
        })?
        .extend(delta_samples.iter().cloned());
    for key in ["run_id", "run_status", "run_failure", "retention"] {
        trace_root.insert(
            key.to_string(),
            delta_trace_root
                .get(key)
                .cloned()
                .unwrap_or(JsonValue::Null),
        );
    }
    let trace = canonical_json(&trace);
    let trace_hash = public_observation_trace_hash(&trace);

    let mut projection = current_projection.clone();
    let projection_root = projection.as_object_mut().ok_or_else(|| {
        "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_BAD_INPUT: current projection은 object여야 합니다"
            .to_string()
    })?;
    if text(projection_root.get("schema")) != PUBLIC_OBSERVATION_TABLE_PROJECTION_SCHEMA {
        return Err(format!(
            "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_SCHEMA: {}",
            text(projection_root.get("schema"))
        ));
    }
    projection_root.remove("projection_hash");
    let delta_projection_root = delta_projection
        .as_object()
        .expect("built delta projection object");
    let delta_run = delta_projection_root
        .get("run")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_RUN_MISSING: run이 필요합니다".to_string()
        })?;
    let current_run = projection_root
        .get("run")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_RUN_MISSING: current run이 필요합니다"
                .to_string()
        })?;
    for key in ["source_identity", "revision_identity", "lock_identity"] {
        if text(current_run.get(key)) != text(delta_run.get(key)) {
            return Err(format!(
                "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_IDENTITY_MISMATCH: {key}"
            ));
        }
    }
    projection_root.insert("run".to_string(), JsonValue::Object(delta_run.clone()));
    projection_root.insert("source_trace_hash".to_string(), json!(trace_hash));

    let display_event_offset = projection_root
        .get("tables")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .map(|table| {
            table
                .get("rows")
                .and_then(JsonValue::as_array)
                .map_or(0, Vec::len)
        })
        .sum::<usize>();
    let tables = projection_root
        .get_mut("tables")
        .and_then(JsonValue::as_array_mut)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_APPEND_PROJECTION_TABLES_MISSING: tables가 필요합니다".to_string()
        })?;
    for delta_table in delta_projection_root
        .get("tables")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
    {
        let delta_columns = delta_table
            .get("columns")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_APPEND_TABLE_COLUMNS_MISSING: columns가 필요합니다"
                    .to_string()
            })?;
        let table_index = tables.iter().position(|table| {
            table.get("columns").and_then(JsonValue::as_array) == Some(delta_columns)
        });
        let target_index = if let Some(index) = table_index {
            index
        } else {
            tables.push(json!({"columns": delta_columns, "rows": []}));
            tables.len() - 1
        };
        let target_rows = tables[target_index]
            .get_mut("rows")
            .and_then(JsonValue::as_array_mut)
            .expect("projection table rows");
        for delta_row in delta_table
            .get("rows")
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
        {
            let mut row = delta_row.clone();
            let local_index = row
                .get("display_event_index")
                .and_then(JsonValue::as_u64)
                .ok_or_else(|| "E_PUBLIC_OBSERVATION_APPEND_DISPLAY_INDEX_MISSING".to_string())?;
            row.as_object_mut().expect("projection row").insert(
                "display_event_index".to_string(),
                json!(display_event_offset + local_index as usize),
            );
            target_rows.push(row);
        }
    }
    let has_rows = tables.iter().any(|table| {
        table
            .get("rows")
            .and_then(JsonValue::as_array)
            .is_some_and(|rows| !rows.is_empty())
    });
    projection_root.insert(
        "presentation_state".to_string(),
        json!(if has_rows { "ready" } else { "no_opportunity" }),
    );
    let projection = seal_public_observation_table_projection(projection);
    Ok((trace, trace_hash, projection))
}

fn trace_identity_key(row: &JsonValue) -> Result<(String, String, String), String> {
    let identity = row
        .get("identity")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_TABLE_IDENTITY_MISSING: name과 source_ref가 필요합니다"
                .to_string()
        })?;
    let name = require_text(identity, "name")?;
    let source_ref = require_text(identity, "source_ref")?;
    Ok((format!("{name}\0{source_ref}"), name, source_ref))
}

fn projected_table_cell(cell: &JsonValue) -> Result<JsonValue, String> {
    let status = text(cell.get("status"));
    let mut projected = json!({"status": status});
    let out = projected.as_object_mut().expect("projected table cell");
    match status.as_str() {
        "present" => {
            let value_canon = cell.get("value_canon").cloned().ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_TABLE_PRESENT_VALUE_MISSING: present cell에는 value_canon이 필요합니다"
                    .to_string()
            })?;
            let value_text = cell.get("value_text").cloned().ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_TABLE_PRESENT_TEXT_MISSING: present cell에는 value_text가 필요합니다"
                    .to_string()
            })?;
            out.insert("value_canon".to_string(), value_canon);
            out.insert("value_text".to_string(), value_text);
            if let Some(dataset_target) = cell.get("dataset_target") {
                out.insert(
                    "dataset_target".to_string(),
                    canonical_dataset_target(dataset_target)?,
                );
            }
        }
        "missing" => {
            let reason = text(cell.get("reason"));
            if !matches!(reason.as_str(), "#계산실패" | "#원천없음" | "#기록손상") {
                return Err(format!(
                    "E_PUBLIC_OBSERVATION_TABLE_MISSING_REASON_UNSUPPORTED: {reason}"
                ));
            }
            out.insert("reason".to_string(), json!(reason));
        }
        "gap" | "pruned" => {
            if cell.get("reason").is_some() {
                return Err(format!(
                    "E_PUBLIC_OBSERVATION_TABLE_REASON_FORBIDDEN: {status}에는 missing reason을 붙일 수 없습니다"
                ));
            }
        }
        _ => {
            return Err(format!(
                "E_PUBLIC_OBSERVATION_TABLE_STATUS_UNSUPPORTED: {status}"
            ))
        }
    }
    Ok(canonical_json(&projected))
}

/// Projects the authoritative public-observation trace into ordered,
/// observation-statement-shaped Workbench/CLI/WASM tables. The returned value
/// is a derived presentation contract; it never replaces or mutates the trace.
pub fn build_public_observation_table_projection(
    trace: &JsonValue,
    expected_run_id: &str,
) -> Result<JsonValue, String> {
    let root = trace.as_object().ok_or_else(|| {
        "E_PUBLIC_OBSERVATION_TABLE_TRACE_BAD_INPUT: trace는 object여야 합니다".to_string()
    })?;
    if text(root.get("schema")) != PUBLIC_OBSERVATION_TRACE_SCHEMA {
        return Err(format!(
            "E_PUBLIC_OBSERVATION_TABLE_TRACE_SCHEMA: {}",
            text(root.get("schema"))
        ));
    }
    let run_id = require_text(root, "run_id")?;
    if expected_run_id.trim().is_empty() || run_id != expected_run_id.trim() {
        return Err(format!(
            "E_PUBLIC_OBSERVATION_TABLE_RUN_MISMATCH: expected={} actual={run_id}",
            expected_run_id.trim()
        ));
    }
    let source_identity = require_text(root, "source_identity")?;
    let revision_identity = require_text(root, "revision_identity")?;
    let run_status = require_text(root, "run_status")?;
    if !matches!(run_status.as_str(), "running" | "completed" | "failed") {
        return Err(format!(
            "E_PUBLIC_OBSERVATION_TABLE_RUN_STATUS_UNSUPPORTED: {run_status}"
        ));
    }
    let source_trace_hash = public_observation_trace_hash(trace);
    let columns = root
        .get("columns")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_TABLE_COLUMNS_MISSING: columns가 필요합니다".to_string()
        })?;
    let samples = root
        .get("samples")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            "E_PUBLIC_OBSERVATION_TABLE_SAMPLES_MISSING: samples가 필요합니다".to_string()
        })?;

    if run_status == "failed" {
        let failure = root
            .get("run_failure")
            .and_then(JsonValue::as_object)
            .ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_TABLE_RUN_FAILURE_MISSING: failed 실행에는 진단이 필요합니다"
                    .to_string()
            })?;
        let has_sample_cells = samples.iter().any(|sample| {
            sample
                .get("cells")
                .and_then(JsonValue::as_array)
                .is_some_and(|cells| !cells.is_empty())
        });
        if !columns.is_empty() || has_sample_cells {
            return Err(
                "E_PUBLIC_OBSERVATION_TABLE_FAILED_RUN_HAS_SAMPLES: 실패 실행을 정상 표본으로 표시할 수 없습니다"
                    .to_string(),
            );
        }
        return Ok(seal_public_observation_table_projection(json!({
            "schema": PUBLIC_OBSERVATION_TABLE_PROJECTION_SCHEMA,
            "source_schema": PUBLIC_OBSERVATION_TRACE_SCHEMA,
            "source_trace_hash": source_trace_hash,
            "presentation_state": "run_failed",
            "run": {
                "run_id": run_id,
                "source_identity": source_identity,
                "revision_identity": revision_identity,
                "lock_identity": text(root.get("lock_identity")),
                "status": run_status,
                "failure": {
                    "code": require_text(failure, "code")?,
                    "message": require_text(failure, "message")?,
                },
            },
            "tables": [],
        })));
    }
    if root
        .get("run_failure")
        .is_some_and(|value| !value.is_null())
    {
        return Err(
            "E_PUBLIC_OBSERVATION_TABLE_RUN_FAILURE_FORBIDDEN: 정상 실행에는 run_failure를 붙일 수 없습니다"
                .to_string(),
        );
    }

    let mut column_index = BTreeMap::new();
    let mut projected_columns = BTreeMap::new();
    for column in columns {
        let (identity_key, name, source_ref) = trace_identity_key(column)?;
        if column_index.insert(identity_key.clone(), ()).is_some() {
            return Err(format!(
                "E_PUBLIC_OBSERVATION_TABLE_COLUMN_DUPLICATE: {name} @ {source_ref}"
            ));
        }
        let dtype = text(column.get("dtype"));
        let format = text(column.get("format"));
        if dtype.is_empty() || format.is_empty() {
            return Err(format!(
                "E_PUBLIC_OBSERVATION_TABLE_COLUMN_TYPE_MISSING: {name} @ {source_ref}"
            ));
        }
        projected_columns.insert(
            identity_key,
            json!({
                "identity": {"name": name, "source_ref": source_ref},
                "label": name,
                "dtype": dtype,
                "unit": text(column.get("unit")),
                "format": format,
            }),
        );
    }

    let mut table_index = BTreeMap::new();
    let mut projected_tables: Vec<JsonValue> = Vec::new();
    let mut display_event_index = 0usize;
    for sample in samples {
        let cells = sample
            .get("cells")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_TABLE_SAMPLE_CELLS_MISSING: cells가 필요합니다".to_string()
            })?;
        if cells.is_empty() {
            continue;
        }
        let committed_madi = sample
            .get("committed_madi")
            .and_then(JsonValue::as_u64)
            .ok_or_else(|| {
                "E_PUBLIC_OBSERVATION_TABLE_SAMPLE_MADI_MISSING: committed_madi가 필요합니다"
                    .to_string()
            })?;
        let state_hash = text(sample.get("state_hash"));
        if state_hash.is_empty() {
            return Err(
                "E_PUBLIC_OBSERVATION_TABLE_SAMPLE_STATE_HASH_MISSING: state_hash가 필요합니다"
                    .to_string(),
            );
        }
        let mut row_cells = Vec::with_capacity(cells.len());
        let mut table_columns = Vec::with_capacity(cells.len());
        let mut signature_parts = Vec::with_capacity(cells.len());
        let mut seen = BTreeSet::new();
        for cell in cells {
            let (identity_key, name, source_ref) = trace_identity_key(cell)?;
            if !seen.insert(identity_key.clone()) {
                return Err(format!(
                    "E_PUBLIC_OBSERVATION_TABLE_SAMPLE_IDENTITY_DUPLICATE: {name} @ {source_ref}"
                ));
            }
            let column = projected_columns.get(&identity_key).ok_or_else(|| {
                format!("E_PUBLIC_OBSERVATION_TABLE_CELL_WITHOUT_COLUMN: {name} @ {source_ref}")
            })?;
            signature_parts.push(identity_key);
            table_columns.push(column.clone());
            row_cells.push(projected_table_cell(cell)?);
        }
        let signature = signature_parts.join("\u{1f}");
        let index = if let Some(index) = table_index.get(&signature).copied() {
            index
        } else {
            let index = projected_tables.len();
            table_index.insert(signature, index);
            projected_tables.push(json!({
                "columns": table_columns,
                "rows": [],
            }));
            index
        };
        let row = json!({
            "display_event_index": display_event_index,
            "committed_madi": committed_madi,
            "state_hash": state_hash,
            "cells": row_cells,
        });
        projected_tables[index]["rows"]
            .as_array_mut()
            .expect("projection table rows")
            .push(row);
        display_event_index += 1;
    }

    let presentation_state = if projected_tables.is_empty() {
        "no_opportunity"
    } else {
        "ready"
    };
    Ok(seal_public_observation_table_projection(json!({
        "schema": PUBLIC_OBSERVATION_TABLE_PROJECTION_SCHEMA,
        "source_schema": PUBLIC_OBSERVATION_TRACE_SCHEMA,
        "source_trace_hash": source_trace_hash,
        "presentation_state": presentation_state,
        "run": {
            "run_id": run_id,
            "source_identity": source_identity,
            "revision_identity": revision_identity,
            "lock_identity": text(root.get("lock_identity")),
            "status": run_status,
            "failure": JsonValue::Null,
        },
        "tables": projected_tables,
    })))
}

fn observation_map(frame: &JsonValue) -> BTreeMap<String, JsonValue> {
    frame
        .get("public_observations")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            observation_identity_key(row)
                .ok()
                .map(|identity| (identity, row.clone()))
        })
        .collect()
}

pub fn diff_committed_frames(a: &JsonValue, b: &JsonValue) -> Result<JsonValue, String> {
    let inherited = Map::new();
    let a = canonical_frame(a, &inherited)?;
    let b = canonical_frame(b, &inherited)?;
    let a_map = observation_map(&a);
    let b_map = observation_map(&b);
    let ids: BTreeSet<String> = a_map.keys().chain(b_map.keys()).cloned().collect();
    let mut changes = Vec::new();
    for identity_key in ids {
        let before = a_map.get(&identity_key);
        let after = b_map.get(&identity_key);
        let kind = match (before, after) {
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (Some(left), Some(right))
                if text(left.get("dtype")) != text(right.get("dtype"))
                    || text(left.get("unit")) != text(right.get("unit")) =>
            {
                "type_or_unit_changed"
            }
            (Some(left), Some(right))
                if text(left.get("value_canon")) != text(right.get("value_canon")) =>
            {
                "value_changed"
            }
            _ => "unchanged",
        };
        if kind != "unchanged" {
            changes.push(json!({
                "identity": before.or(after).map(observation_identity).unwrap_or(JsonValue::Null),
                "kind": kind,
                "before": before.cloned().unwrap_or(JsonValue::Null),
                "after": after.cloned().unwrap_or(JsonValue::Null),
            }));
        }
    }
    let same_source = text(a.get("source_identity")) == text(b.get("source_identity"))
        && text(a.get("revision_identity")) == text(b.get("revision_identity"));
    let diff = json!({
        "schema": DEBUG_DIFF_SCHEMA,
        "a_madi": a.get("committed_madi").cloned().unwrap_or(JsonValue::Null),
        "b_madi": b.get("committed_madi").cloned().unwrap_or(JsonValue::Null),
        "same_run": text(a.get("run_id")) == text(b.get("run_id")),
        "same_source_revision": same_source,
        "input_changed": a.get("input_snapshot") != b.get("input_snapshot"),
        "input_before": a.get("input_snapshot").cloned().unwrap_or(JsonValue::Null),
        "input_after": b.get("input_snapshot").cloned().unwrap_or(JsonValue::Null),
        "state_hash_changed": text(a.get("state_hash")) != text(b.get("state_hash")),
        "state_hash_before": text(a.get("state_hash")),
        "state_hash_after": text(b.get("state_hash")),
        "observation_changes": changes,
    });
    Ok(canonical_json(&diff))
}

fn validate_plan(
    plan: &JsonValue,
    actual_source_sha256: Option<&str>,
) -> Result<JsonValue, String> {
    let obj = plan
        .as_object()
        .ok_or_else(|| "E_DEBUG_PLAN_BAD_INPUT: plan은 object여야 합니다".to_string())?;
    let run_id = require_text(obj, "run_id")?;
    let source_identity = require_text(obj, "source_identity")?;
    let revision_identity = require_text(obj, "revision_identity")?;
    let source_bytes_sha256 = require_text(obj, "source_bytes_sha256")?;
    if let Some(actual) = actual_source_sha256 {
        if source_bytes_sha256 != actual {
            return Err(
                "E_DEBUG_PLAN_SOURCE_MISMATCH: plan과 실행 DDN source가 다릅니다".to_string(),
            );
        }
    }
    let conditions = obj
        .get("conditions")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "E_DEBUG_PLAN_CONDITIONS_MISSING: conditions가 필요합니다".to_string())?;
    if conditions.is_empty() {
        return Err("E_DEBUG_PLAN_CONDITIONS_EMPTY: 한 개 이상의 조건이 필요합니다".to_string());
    }
    let mut condition_ids = BTreeSet::new();
    let mut normalized = Vec::with_capacity(conditions.len());
    for (index, condition) in conditions.iter().enumerate() {
        let row = condition.as_object().ok_or_else(|| {
            "E_DEBUG_CONDITION_BAD_ROW: condition은 object여야 합니다".to_string()
        })?;
        let condition_id = require_text(row, "condition_id")?;
        if !condition_ids.insert(condition_id.clone()) {
            return Err(format!("E_DEBUG_CONDITION_ID_DUPLICATE: {condition_id}"));
        }
        let kind = require_text(row, "kind")?;
        match kind.as_str() {
            "exact_madi" => {
                let madi = row.get("madi").and_then(JsonValue::as_u64).ok_or_else(|| {
                    "E_DEBUG_CONDITION_MADI_MISSING: exact_madi에는 madi가 필요합니다".to_string()
                })?;
                normalized.push(json!({"declaration_index": index, "condition_id": condition_id, "kind": kind, "madi": madi}));
            }
            "observation_changed" => {
                let binding = row
                    .get("binding")
                    .and_then(JsonValue::as_object)
                    .ok_or_else(|| "E_DEBUG_CONDITION_BINDING_MISSING: observation_changed에는 binding이 필요합니다".to_string())?;
                if binding.contains_key("observation_id") {
                    return Err(
                        "E_BOIM_SYNTHETIC_ID_REMOVED: debug binding은 DDN 이름과 source_ref를 사용합니다"
                            .to_string(),
                    );
                }
                for key in ["name", "dtype", "source_ref"] {
                    require_text(binding, key)?;
                }
                normalized.push(json!({
                    "declaration_index": index,
                    "condition_id": condition_id,
                    "kind": kind,
                    "binding": {
                        "name": text(binding.get("name")),
                        "dtype": text(binding.get("dtype")),
                        "unit": text(binding.get("unit")),
                        "source_ref": text(binding.get("source_ref")),
                    }
                }));
            }
            _ => return Err(format!("E_DEBUG_CONDITION_UNSUPPORTED: {kind}")),
        }
    }
    let normalized = canonical_json(&json!({
        "schema": DEBUG_PLAN_SCHEMA,
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "lock_identity": text(obj.get("lock_identity")),
        "source_bytes_sha256": source_bytes_sha256,
        "combine": "OR",
        "conditions": normalized,
    }));
    Ok(json!({"plan": normalized, "plan_hash": sha256_json(&normalized)}))
}

#[derive(Clone, Debug)]
pub struct DebugEvaluator {
    plan: JsonValue,
    plan_hash: String,
    previous: BTreeMap<String, JsonValue>,
}

impl DebugEvaluator {
    pub fn from_plan(plan: &JsonValue, actual_source_sha256: Option<&str>) -> Result<Self, String> {
        let normalized = validate_plan(plan, actual_source_sha256)?;
        Ok(Self {
            plan: normalized.get("plan").cloned().unwrap_or(JsonValue::Null),
            plan_hash: text(normalized.get("plan_hash")),
            previous: BTreeMap::new(),
        })
    }

    pub fn plan_hash(&self) -> &str {
        &self.plan_hash
    }

    pub fn reset_history(&mut self) {
        self.previous.clear();
    }

    pub fn evaluate_frame(&mut self, frame: &JsonValue) -> Result<JsonValue, String> {
        let inherited = self.plan.as_object().cloned().unwrap_or_default();
        let frame = canonical_frame(frame, &inherited)?;
        for key in ["run_id", "source_identity", "revision_identity"] {
            if text(frame.get(key)) != text(self.plan.get(key)) {
                return Err(format!("E_DEBUG_FRAME_IDENTITY_MISMATCH: {key}"));
            }
        }
        let madi = frame
            .get("committed_madi")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0);
        let current = observation_map(&frame);
        let mut reasons = Vec::new();
        for condition in self
            .plan
            .get("conditions")
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
        {
            let kind = text(condition.get("kind"));
            let condition_id = text(condition.get("condition_id"));
            if kind == "exact_madi"
                && condition.get("madi").and_then(JsonValue::as_u64) == Some(madi)
            {
                reasons.push(
                    json!({"condition_id": condition_id, "kind": kind, "committed_madi": madi}),
                );
            } else if kind == "observation_changed" {
                let binding = condition.get("binding").unwrap_or(&JsonValue::Null);
                let identity = observation_identity_key(binding)?;
                let now = current.get(&identity);
                let before = self.previous.get(&identity);
                if let Some(now) = now {
                    let binding_matches = text(now.get("dtype")) == text(binding.get("dtype"))
                        && text(now.get("unit")) == text(binding.get("unit"))
                        && text(now.get("source_ref")) == text(binding.get("source_ref"));
                    if !binding_matches {
                        return Err(format!(
                            "E_DEBUG_OBSERVATION_BINDING_STALE: {} @ {}",
                            text(binding.get("name")),
                            text(binding.get("source_ref"))
                        ));
                    }
                    if let Some(before) = before {
                        if text(before.get("value_canon")) != text(now.get("value_canon")) {
                            reasons.push(json!({
                                "condition_id": condition_id,
                                "kind": kind,
                                "committed_madi": madi,
                                "identity": observation_identity(now),
                                "before": before,
                                "after": now,
                            }));
                        }
                    }
                } else if self.previous.contains_key(&identity) {
                    return Err(format!(
                        "E_DEBUG_OBSERVATION_REMOVED: {} @ {}",
                        text(binding.get("name")),
                        text(binding.get("source_ref"))
                    ));
                }
            }
        }
        self.previous = current;
        Ok(canonical_json(&json!({
            "schema": DEBUG_EVIDENCE_SCHEMA,
            "triggered": !reasons.is_empty(),
            "committed_madi": madi,
            "state_hash": text(frame.get("state_hash")),
            "debug_plan_hash": self.plan_hash,
            "reasons": reasons,
        })))
    }
}

pub fn evaluate_sealed_sequence(input: &JsonValue) -> Result<JsonValue, String> {
    let plan = input.get("plan").unwrap_or(&JsonValue::Null);
    let frames = input
        .get("frames")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "E_DEBUG_REPLAY_FRAMES_MISSING: frames가 필요합니다".to_string())?;
    let mut evaluator = DebugEvaluator::from_plan(plan, None)?;
    let mut evaluated = Vec::new();
    let mut pauses = Vec::new();
    for frame in frames {
        let evidence = evaluator.evaluate_frame(frame)?;
        if evidence.get("triggered").and_then(JsonValue::as_bool) == Some(true) {
            pauses.push(evidence.clone());
        }
        evaluated.push(evidence);
    }
    Ok(canonical_json(&json!({
        "schema": "seamgrim.ddn_sealed_debug_replay.candidate.v1",
        "debug_plan_hash": evaluator.plan_hash(),
        "evaluated_prefix_length": frames.len(),
        "evaluated": evaluated,
        "pause_sequence": pauses,
    })))
}

pub fn deterministic_debug_operation(
    operation: &str,
    input: &JsonValue,
) -> Result<JsonValue, String> {
    match operation.trim() {
        "trace" => build_public_observation_trace(input),
        "diff" => diff_committed_frames(
            input.get("a").unwrap_or(&JsonValue::Null),
            input.get("b").unwrap_or(&JsonValue::Null),
        ),
        "plan" => validate_plan(input, None),
        "replay" => evaluate_sealed_sequence(input),
        other => Err(format!("E_DEBUG_OPERATION_UNSUPPORTED: {other}")),
    }
}

/// Runs the existing deterministic-debug file workflow through shared tool
/// authority. `Ok(Some(_))` is stdout output; `Ok(None)` means an artifact was
/// atomically published at `out_path`.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_deterministic_debug_file(
    operation: &str,
    input_path: &Path,
    out_path: Option<&Path>,
) -> Result<Option<String>, String> {
    let input_text = fs::read_to_string(input_path)
        .map_err(|error| format!("E_DEBUG_INPUT_READ: {} ({error})", input_path.display()))?;
    let input_json: JsonValue = serde_json::from_str(&input_text)
        .map_err(|error| format!("E_DEBUG_JSON_INVALID: {error}"))?;
    let value = deterministic_debug_operation(operation, &input_json)?;
    let output = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("E_DEBUG_OUTPUT_SERIALIZE: {error}"))?
        + "\n";

    let Some(out_path) = out_path else {
        return Ok(Some(output));
    };
    if out_path.exists() && !out_path.is_file() {
        return Err(format!(
            "E_DEBUG_OUTPUT_TARGET_NOT_FILE: {}",
            out_path.display()
        ));
    }
    write_text_artifact_atomic(out_path, &output)
        .map_err(|error| format!("E_DEBUG_OUTPUT_WRITE: {} ({error})", out_path.display()))?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FILE_WORKFLOW_NONCE: AtomicU64 = AtomicU64::new(0);

    fn file_workflow_dir(label: &str) -> PathBuf {
        let nonce = FILE_WORKFLOW_NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_deterministic_debug_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    fn observation(name: &str, source_ref: &str, value: &str) -> JsonValue {
        json!({
            "name": name,
            "dtype": "fixed64",
            "unit": "",
            "format": "ddn-canonical",
            "source_ref": source_ref,
            "value_canon": value,
            "value_text": value,
            "starts_observation_event": true,
        })
    }

    fn continuation_observation(name: &str, source_ref: &str, value: &str) -> JsonValue {
        let mut row = observation(name, source_ref, value);
        row["starts_observation_event"] = json!(false);
        row
    }

    fn frame(madi: u64, value: &str) -> JsonValue {
        json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "committed_madi": madi,
            "state_hash": format!("state-{madi}"),
            "input_snapshot": {"tick_id": madi},
            "public_observations": [observation("값", "main.ddn#bytes=10:13", value)],
        })
    }

    fn plan() -> JsonValue {
        json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "source_bytes_sha256": "abc",
            "conditions": [
                {"condition_id": "c1", "kind": "observation_changed", "binding": {
                    "name": "값", "dtype": "fixed64", "unit": "", "source_ref": "main.ddn#bytes=10:13"
                }},
                {"condition_id": "c2", "kind": "exact_madi", "madi": 2}
            ]
        })
    }

    #[test]
    fn file_workflow_stdout_and_atomic_artifact_are_identical() {
        let dir = file_workflow_dir("atomic");
        fs::create_dir_all(&dir).expect("create test dir");
        let input = dir.join("input.json");
        let output = dir.join("nested/result.detjson");
        fs::write(&input, serde_json::to_vec(&plan()).expect("serialize plan"))
            .expect("write input");

        let stdout = run_deterministic_debug_file("plan", &input, None)
            .expect("stdout workflow")
            .expect("stdout text");
        assert_eq!(
            run_deterministic_debug_file("plan", &input, Some(&output)),
            Ok(None)
        );
        assert_eq!(fs::read_to_string(&output).expect("read artifact"), stdout);
        assert_eq!(
            fs::read_dir(output.parent().expect("parent"))
                .expect("read output dir")
                .count(),
            1
        );

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn file_workflow_rejects_directory_target_without_publish() {
        let dir = file_workflow_dir("target_kind");
        fs::create_dir_all(&dir).expect("create test dir");
        let input = dir.join("input.json");
        let target = dir.join("blocked.detjson");
        fs::write(&input, serde_json::to_vec(&plan()).expect("serialize plan"))
            .expect("write input");
        fs::create_dir(&target).expect("create blocking directory");

        let error = run_deterministic_debug_file("plan", &input, Some(&target))
            .expect_err("directory target must fail closed");
        assert!(error.contains("E_DEBUG_OUTPUT_TARGET_NOT_FILE"));
        assert_eq!(fs::read_dir(&target).expect("read blocker").count(), 0);

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn trace_is_canonical_preserves_order_and_does_not_infer_missing_madi_status() {
        let input = json!({
            "project_id": "display-only",
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [frame(2, "2"), frame(0, "0")]
        });
        let first = build_public_observation_trace(&input).expect("trace");
        let second = build_public_observation_trace(&input).expect("trace");
        assert_eq!(first, second);
        assert_eq!(
            first["trace"]["schema"],
            json!(PUBLIC_OBSERVATION_TRACE_SCHEMA)
        );
        assert_eq!(
            first["trace"]["columns"][0]["identity"]["name"],
            json!("값")
        );
        assert_eq!(first["trace"]["samples"][0]["committed_madi"], json!(0));
        assert_eq!(first["trace"]["samples"][1]["committed_madi"], json!(2));
        assert!(first["trace"].get("gaps").is_none());
        assert!(text(first.get("trace_hash")).starts_with("sha256:"));
    }

    #[test]
    fn incremental_observation_surface_matches_full_rebuild_exactly() {
        let root = |run_id: &str, frames: Vec<JsonValue>| {
            json!({
                "project_id": "",
                "run_id": run_id,
                "source_identity": "source-1",
                "revision_identity": "revision-1",
                "run_status": "completed",
                "frames": frames,
            })
        };
        let wasm_frame = |madi: u64, value: &str| {
            let mut row = frame(madi, value);
            for key in ["run_id", "source_identity", "revision_identity"] {
                row.as_object_mut().expect("frame object").remove(key);
            }
            row
        };

        let empty =
            build_public_observation_trace(&root("run-0", Vec::new())).expect("empty trace");
        let mut incremental_trace = empty["trace"].clone();
        let mut incremental_projection =
            build_public_observation_table_projection(&incremental_trace, "run-0")
                .expect("empty projection");
        let mut all_frames = Vec::new();

        for (madi, value) in [(0, "0"), (1, "1"), (2, "2")] {
            let next = wasm_frame(madi, value);
            all_frames.push(next.clone());
            let run_id = format!("run-{}", all_frames.len());
            let (next_trace, incremental_hash, next_projection) =
                append_public_observation_surface(
                    &incremental_trace,
                    &incremental_projection,
                    &root(&run_id, vec![next]),
                )
                .expect("incremental append");
            incremental_trace = next_trace;
            incremental_projection = next_projection;

            let full = build_public_observation_trace(&root(&run_id, all_frames.clone()))
                .expect("full trace");
            let full_projection =
                build_public_observation_table_projection(&full["trace"], &run_id)
                    .expect("full projection");
            assert_eq!(incremental_trace, full["trace"]);
            assert_eq!(incremental_hash, text(full.get("trace_hash")));
            assert_eq!(incremental_projection, full_projection);
        }
    }

    #[test]
    fn repeated_statement_emissions_become_ordered_rows_without_synthetic_identity() {
        let mut repeated = frame(4, "unused");
        repeated["public_observations"] = json!([
            observation("t", "main.ddn#bytes=100:101", "0"),
            continuation_observation("온도", "main.ddn#bytes=103:109", "80"),
            observation("t", "main.ddn#bytes=100:101", "1"),
            continuation_observation("온도", "main.ddn#bytes=103:109", "76")
        ]);
        let result = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [repeated]
        }))
        .expect("repeated statement emissions");
        let trace = &result["trace"];
        assert_eq!(trace["columns"].as_array().map(Vec::len), Some(2));
        assert_eq!(trace["samples"].as_array().map(Vec::len), Some(2));
        assert_eq!(trace["samples"][0]["committed_madi"], json!(4));
        assert_eq!(trace["samples"][1]["committed_madi"], json!(4));
        assert_eq!(trace["samples"][0]["cells"][0]["value_canon"], json!("0"));
        assert_eq!(trace["samples"][1]["cells"][0]["value_canon"], json!("1"));
        assert!(trace["columns"][0]["identity"]
            .get("observation_id")
            .is_none());
    }

    #[test]
    fn sparse_statuses_are_explicit_missing_requires_reason_and_no_interpolation_exists() {
        let mut sparse = frame(1, "1");
        sparse["public_observations"] = json!([]);
        sparse["public_observation_statuses"] = json!([
            {
                "name": "값",
                "source_ref": "main.ddn#bytes=10:13",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "gap",
                "starts_observation_event": true
            },
            {
                "name": "다른값",
                "source_ref": "main.ddn#bytes=20:29",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "missing",
                "reason": "#계산실패",
                "starts_observation_event": false
            },
            {
                "name": "지난값",
                "source_ref": "main.ddn#bytes=30:39",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "pruned",
                "starts_observation_event": false
            }
        ]);
        let trace = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [sparse]
        }))
        .expect("sparse trace");
        let cells = trace["trace"]["samples"][0]["cells"]
            .as_array()
            .expect("cells");
        assert_eq!(
            cells
                .iter()
                .map(|cell| text(cell.get("status")))
                .collect::<Vec<_>>(),
            vec!["gap", "missing", "pruned"]
        );
        assert_eq!(cells[1]["reason"], json!("#계산실패"));
        assert!(trace["trace"].get("interpolation").is_none());

        let mut bad = frame(1, "1");
        bad["public_observations"] = json!([]);
        bad["public_observation_statuses"] = json!([{
            "name": "값",
            "source_ref": "main.ddn#bytes=10:13",
            "status": "missing",
            "starts_observation_event": true
        }]);
        let error = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [bad]
        }))
        .expect_err("missing reason required");
        assert!(error.contains("REQUIRED_IDENTITY"));
    }

    #[test]
    fn table_projection_preserves_authored_columns_events_and_same_name_source_refs() {
        let mut repeated = frame(4, "unused");
        repeated["public_observations"] = json!([
            observation("값", "app/main.ddn#bytes=100:103", "15"),
            continuation_observation("증가", "app/main.ddn#bytes=120:126", "2"),
            observation("값", "app/main.ddn#bytes=140:143", "2"),
            continuation_observation("다른", "app/main.ddn#bytes=150:156", "3"),
            observation("값", "app/main.ddn#bytes=100:103", "16"),
            continuation_observation("증가", "app/main.ddn#bytes=120:126", "2")
        ]);
        let bundle = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [repeated]
        }))
        .expect("trace");
        let trace = bundle["trace"].clone();
        let before = trace.clone();
        let trace_hash_before = public_observation_trace_hash(&trace);
        let table = build_public_observation_table_projection(&trace, "run-1").expect("table");

        assert_eq!(
            table["schema"],
            json!(PUBLIC_OBSERVATION_TABLE_PROJECTION_SCHEMA)
        );
        assert_eq!(table["presentation_state"], json!("ready"));
        assert_eq!(
            table["tables"][0]["columns"]
                .as_array()
                .expect("columns")
                .iter()
                .map(|column| text(column.pointer("/identity/name")))
                .collect::<Vec<_>>(),
            vec!["값", "증가"]
        );
        let same_name_refs = table["tables"]
            .as_array()
            .expect("tables")
            .iter()
            .map(|table| text(table.pointer("/columns/0/identity/source_ref")))
            .collect::<Vec<_>>();
        assert_eq!(
            same_name_refs,
            vec!["app/main.ddn#bytes=100:103", "app/main.ddn#bytes=140:143"]
        );
        assert_eq!(table["tables"].as_array().map(Vec::len), Some(2));
        let rows = table["tables"][0]["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["display_event_index"], json!(0));
        assert_eq!(rows[1]["display_event_index"], json!(2));
        assert_eq!(rows[0]["committed_madi"], json!(4));
        assert_eq!(rows[1]["committed_madi"], json!(4));
        assert_eq!(rows[0]["cells"][0]["value_text"], json!("15"));
        assert_eq!(rows[0]["cells"][1]["value_text"], json!("2"));
        assert_eq!(rows[1]["cells"][0]["value_text"], json!("16"));
        assert_eq!(rows[1]["cells"][1]["value_text"], json!("2"));
        assert_eq!(
            table["tables"][1]["rows"][0]["display_event_index"],
            json!(1)
        );
        assert_eq!(
            table["tables"][1]["rows"][0]["cells"][1]["value_text"],
            json!("3")
        );
        assert_eq!(table["source_trace_hash"], json!(trace_hash_before));
        let projection_hash = text(table.get("projection_hash"));
        assert!(projection_hash.starts_with("sha256:"));
        assert_eq!(
            projection_hash,
            public_observation_table_projection_hash(&table)
        );
        let mut synthetic_column_attack = table.clone();
        synthetic_column_attack["tables"][0]["columns"]
            .as_array_mut()
            .expect("columns")
            .push(json!({
                "identity": {"name": "output_9", "source_ref": "synthetic"},
                "label": "output_9",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical"
            }));
        assert_ne!(
            text(synthetic_column_attack.get("projection_hash")),
            public_observation_table_projection_hash(&synthetic_column_attack)
        );
        assert_eq!(trace, before);
        assert_eq!(public_observation_trace_hash(&trace), trace_hash_before);
        assert!(serde_json::to_string(&table)
            .expect("json")
            .find("project.obs.")
            .is_none());
        assert!(serde_json::to_string(&table)
            .expect("json")
            .find("output_")
            .is_none());
    }

    #[test]
    fn table_projection_ignores_project_and_sidecar_semantics_without_reordering_fields() {
        let mut authored = frame(3, "unused");
        authored["public_observations"] = json!([
            observation("x", "app/main.ddn#bytes=100:101", "1"),
            continuation_observation("y", "app/main.ddn#bytes=110:111", "2"),
            continuation_observation("vx", "app/main.ddn#bytes=120:122", "3"),
            continuation_observation("vy", "app/main.ddn#bytes=130:132", "4")
        ]);
        let base = json!({
            "project_id": "project-a",
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [authored]
        });
        let base_bundle = build_public_observation_trace(&base).expect("base trace");
        let base_table = build_public_observation_table_projection(&base_bundle["trace"], "run-1")
            .expect("base table");

        let mut attacked = base;
        attacked["project_id"] = json!("project-b");
        attacked["sidecar"] = json!({
            "units": {"x": "m", "y": "m", "vx": "m/s", "vy": "m/s"},
            "roles": {"x": "x-axis", "y": "y-axis"},
            "primary_view": "graph",
            "field_order": ["vy", "vx", "y", "x"]
        });
        let attacked_bundle = build_public_observation_trace(&attacked).expect("attacked trace");
        let attacked_table =
            build_public_observation_table_projection(&attacked_bundle["trace"], "run-1")
                .expect("attacked table");

        assert_ne!(
            base_bundle["trace_hash"], attacked_bundle["trace_hash"],
            "project lineage change must remain visible in the trace hash"
        );
        assert_eq!(
            base_table["tables"], attacked_table["tables"],
            "project id or sidecar changed authored table semantics"
        );
        let columns = attacked_table["tables"][0]["columns"]
            .as_array()
            .expect("columns");
        assert_eq!(
            columns
                .iter()
                .map(|column| text(column.pointer("/identity/name")))
                .collect::<Vec<_>>(),
            vec!["x", "y", "vx", "vy"]
        );
        assert!(columns
            .iter()
            .all(|column| text(column.get("unit")).is_empty()));
        assert!(attacked_bundle["trace"].get("sidecar").is_none());
    }

    #[test]
    fn table_projection_keeps_sparse_states_and_rejects_reason_forgery() {
        let mut sparse = frame(7, "unused");
        sparse["public_observations"] = json!([]);
        sparse["public_observation_statuses"] = json!([
            {
                "name": "틈",
                "source_ref": "main.ddn#bytes=10:13",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "gap",
                "starts_observation_event": true
            },
            {
                "name": "빠짐",
                "source_ref": "main.ddn#bytes=20:26",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "missing",
                "reason": "#원천없음",
                "starts_observation_event": false
            },
            {
                "name": "잘림",
                "source_ref": "main.ddn#bytes=30:36",
                "dtype": "fixed64",
                "unit": "",
                "format": "ddn-canonical",
                "status": "pruned",
                "starts_observation_event": false
            }
        ]);
        let bundle = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [sparse]
        }))
        .expect("sparse trace");
        let table = build_public_observation_table_projection(&bundle["trace"], "run-1")
            .expect("sparse table");
        let cells = table["tables"][0]["rows"][0]["cells"]
            .as_array()
            .expect("cells");
        assert_eq!(
            cells
                .iter()
                .map(|cell| text(cell.get("status")))
                .collect::<Vec<_>>(),
            vec!["gap", "missing", "pruned"]
        );
        assert!(cells[0].get("reason").is_none());
        assert_eq!(cells[1]["reason"], json!("#원천없음"));
        assert!(cells[2].get("reason").is_none());

        let mut forged = bundle["trace"].clone();
        forged["samples"][0]["cells"][1]["reason"] = json!("#임의이유");
        let error = build_public_observation_table_projection(&forged, "run-1")
            .expect_err("unsupported reason must fail");
        assert!(error.contains("MISSING_REASON_UNSUPPORTED"));

        let mut reason_on_gap = bundle["trace"].clone();
        reason_on_gap["samples"][0]["cells"][0]["reason"] = json!("#원천없음");
        let error = build_public_observation_table_projection(&reason_on_gap, "run-1")
            .expect_err("gap reason must fail");
        assert!(error.contains("REASON_FORBIDDEN"));
    }

    #[test]
    fn table_projection_preserves_all_three_approved_missing_reasons() {
        for reason in ["#계산실패", "#원천없음", "#기록손상"] {
            let projected = projected_table_cell(&json!({
                "status": "missing",
                "reason": reason,
            }))
            .expect("approved missing reason");
            assert_eq!(projected["status"], json!("missing"));
            assert_eq!(projected["reason"], json!(reason));
            assert!(projected.get("value_canon").is_none());
            assert!(projected.get("value_text").is_none());
        }
        let error = projected_table_cell(&json!({
            "status": "missing",
            "reason": "#임의이유",
        }))
        .expect_err("unsupported missing reason");
        assert!(error.contains("MISSING_REASON_UNSUPPORTED"));
    }

    #[test]
    fn table_projection_separates_no_opportunity_failure_and_cross_run() {
        let mut empty = frame(0, "unused");
        empty["public_observations"] = json!([]);
        let empty_bundle = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [empty]
        }))
        .expect("empty trace");
        let empty_table =
            build_public_observation_table_projection(&empty_bundle["trace"], "run-1")
                .expect("empty table");
        assert_eq!(empty_table["presentation_state"], json!("no_opportunity"));
        assert_eq!(empty_table["tables"], json!([]));

        let failed_bundle = build_public_observation_trace(&json!({
            "run_id": "run-failed",
            "source_identity": "source-failed",
            "revision_identity": "revision-failed",
            "run_status": "failed",
            "run_failure": {"code": "E_RUN", "message": "실행 실패"},
            "frames": []
        }))
        .expect("failed trace");
        let failed_table =
            build_public_observation_table_projection(&failed_bundle["trace"], "run-failed")
                .expect("failed table");
        assert_eq!(failed_table["presentation_state"], json!("run_failed"));
        assert_eq!(failed_table["run"]["failure"]["code"], json!("E_RUN"));
        assert_eq!(failed_table["tables"], json!([]));

        let mismatch =
            build_public_observation_table_projection(&empty_bundle["trace"], "run-other")
                .expect_err("cross-run projection must fail");
        assert!(mismatch.contains("RUN_MISMATCH"));

        let mut failed_with_sample = failed_bundle["trace"].clone();
        failed_with_sample["columns"] = json!([{
            "identity": {"name": "값", "source_ref": "main.ddn#bytes=1:4"},
            "dtype": "fixed64",
            "unit": "",
            "format": "ddn-canonical"
        }]);
        failed_with_sample["samples"] = json!([{
            "committed_madi": 0,
            "state_hash": "state-failed",
            "cells": [{
                "identity": {"name": "값", "source_ref": "main.ddn#bytes=1:4"},
                "status": "present",
                "value_canon": "1",
                "value_text": "1"
            }]
        }]);
        let error = build_public_observation_table_projection(&failed_with_sample, "run-failed")
            .expect_err("failed run samples must fail");
        assert!(error.contains("FAILED_RUN_HAS_SAMPLES"));
    }

    #[test]
    fn diff_uses_public_observations_only() {
        let diff = diff_committed_frames(&frame(0, "0"), &frame(1, "1")).expect("diff");
        assert_eq!(
            diff["observation_changes"][0]["kind"],
            json!("value_changed")
        );
        assert_eq!(diff["state_hash_changed"], json!(true));
    }

    #[test]
    fn live_and_sealed_prefix_have_same_ordered_pause_sequence() {
        let frames = vec![frame(0, "0"), frame(1, "1"), frame(2, "2")];
        let replay =
            evaluate_sealed_sequence(&json!({"plan": plan(), "frames": frames})).expect("replay");
        assert_eq!(replay["pause_sequence"].as_array().map(Vec::len), Some(2));
        assert_eq!(replay["pause_sequence"][0]["committed_madi"], json!(1));
        assert_eq!(
            replay["pause_sequence"][1]["reasons"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn source_binding_and_unsupported_predicate_fail_closed() {
        assert!(DebugEvaluator::from_plan(&plan(), Some("other"))
            .unwrap_err()
            .contains("SOURCE_MISMATCH"));
        let mut unsupported = plan();
        unsupported["conditions"][0]["kind"] = json!("zero_crossing");
        assert!(DebugEvaluator::from_plan(&unsupported, None)
            .unwrap_err()
            .contains("UNSUPPORTED"));
    }

    #[test]
    fn cross_run_refusal_and_run_failure_are_not_sample_cells() {
        let mut mixed = frame(1, "1");
        mixed["run_id"] = json!("run-2");
        let error = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "completed",
            "frames": [frame(0, "0"), mixed]
        }))
        .expect_err("mixed run refused");
        assert!(error.contains("IDENTITY_MIXED"));

        let failed = build_public_observation_trace(&json!({
            "run_id": "run-1",
            "source_identity": "source-1",
            "revision_identity": "revision-1",
            "run_status": "failed",
            "run_failure": {"code": "E_RUNTIME", "message": "실행 실패"},
            "frames": []
        }))
        .expect("failed trace");
        assert_eq!(failed["trace"]["run_status"], json!("failed"));
        assert_eq!(failed["trace"]["run_failure"]["code"], json!("E_RUNTIME"));
        assert_eq!(failed["trace"]["samples"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn trace_hash_excludes_state_hash_but_includes_observation_value() {
        let base = |state_hash: &str, value: &str| {
            let mut row = frame(0, value);
            row["state_hash"] = json!(state_hash);
            build_public_observation_trace(&json!({
                "run_id": "run-1",
                "source_identity": "source-1",
                "revision_identity": "revision-1",
                "run_status": "completed",
                "frames": [row]
            }))
            .expect("trace")
        };
        assert_eq!(
            base("state-a", "1")["trace_hash"],
            base("state-b", "1")["trace_hash"]
        );
        assert_ne!(
            base("state-a", "1")["trace_hash"],
            base("state-a", "2")["trace_hash"]
        );
    }

    #[test]
    fn synthetic_observation_ids_are_rejected_without_migration_fallback() {
        let error = canonical_public_observation_events([json!({
            "observation_id": "project.obs.1",
            "name": "값",
            "source_ref": "main.ddn#bytes=10:13",
            "dtype": "fixed64",
            "value_canon": "1"
        })])
        .expect_err("synthetic id rejected");
        assert!(error.contains("SYNTHETIC_ID_REMOVED"));
    }
}
