use ddonirang_core::platform::{RecordFieldType, ResourceMapEntry, ResourceValue, RegisteredAtomValueV1};
use std::collections::BTreeMap;
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue};
use serde_json::{json, Value as JsonValue};
mod bytes_value;
mod tuple_value;
#[allow(unused_imports)] // Native-only builds do not export a WASM snapshot.
pub(crate) use bytes_value::{serialize_resource_snapshot_json, write_compact_json, write_pretty_json};

use crate::fixed64_boundary::reject_binary_json_number;

// RD-674/RD-682/RD-683: one shared issuer/verifier for all product surfaces.
mod tensor_frame;
pub(crate) mod project_runtime_attachment;
mod complete_state_frame;
pub(crate) mod subject_execution;
pub(crate) mod resumable_complete_state;
mod complete_state_tables;
mod saved_resource_value;
#[cfg(test)]
pub(crate) use saved_resource_value::check_actual_r5_nominal_catalog_vectors;
mod resource_snapshot;
pub(crate) mod runtime_checkpoint;
pub(crate) use resource_snapshot::{restore_world_resources_snapshot,parse_param_overrides};
pub(crate) use resource_snapshot::restore_world_resources_with_tuple_plan;
pub(crate) use resource_snapshot::stage_resource_snapshot_with_complete_frame;
pub(crate) use saved_resource_value::check_saved_type as validate_saved_value_type;
pub(crate) use saved_resource_value::decode_recorded_field_type;
pub(crate) use saved_resource_value::decode_result_output_field_type;
pub(crate) fn read_registered_nominal_slots(
    value:&std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>,
    catalog:&crate::authored_project::registration::SavedNominalCatalogV1,
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
)->Result<BTreeMap<[u8;32],ResourceValue>,crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1>{
    saved_resource_value::decode_nominal_slots(value,catalog,references,contracts,2)
}
pub(crate) use tuple_value::requires_source_type as saved_resource_contains_tuple;
pub(crate) use complete_state_frame::{check_complete_state_frame_v1, CompleteStateFrameV1, CompleteStateValueTablesV1};
pub(crate) use complete_state_frame::{check_resumable_complete_state_layout_v2, ResumableCompleteStateLayoutV2};

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
        let selected_row = tensor_select(&old, &[TensorSelector::Index(1)]).unwrap();
        let Value::Tensor(row) = &selected_row else { panic!() };
        assert_eq!(row.shape(), &[3]);
        assert_eq!(row.elements(), &(3..6).map(Fixed64::from_i64).collect::<Vec<_>>());
        assert_eq!(tensor_select(&old, &[TensorSelector::Index(1),TensorSelector::Index(2)]).unwrap(), Value::Fixed64(Fixed64::from_i64(5)));
        let selected_column = tensor_select(&old, &[TensorSelector::Full,TensorSelector::Range { start: 1, end: 2 }]).unwrap();
        let Value::Tensor(column) = &selected_column else { panic!() };
        assert_eq!(column.shape(), &[2,1]);
        assert_eq!(column.elements(), &[Fixed64::from_i64(1),Fixed64::from_i64(4)]);
        let selected_empty = tensor_select(&old, &[TensorSelector::Range { start: 2, end: 2 }]).unwrap();
        let Value::Tensor(empty) = &selected_empty else { panic!() };
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
    // F5-GA[WIRE_SELECTION] axes/coordinates share one fixed length; zero retained extents return early. Every emitted offset is checked against elements.len, and an empty result shape emits exactly one scalar offset. Replacement clones that same element length.
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
    // F5-GA[WIRE_TENSOR_TAG] verify_runtime inspects the saved tag before slicing [1..]; the restore counterpart receives an AdmittedTensor built from an explicitly tagged and inspected saved frame.
    verify_runtime_tensor(value)?;
    Ok(ResourceValue::BasicTensorV1(value.saved_bytes()[1..].to_vec()))
}
pub(crate) fn tensor_from_carrier(carrier: &[u8]) -> Result<ddonirang_lang::runtime::TensorValue, String> {
    tensor_frame::from_carrier(carrier).and_then(|v| v.into_runtime()).map_err(|e| format!("Tensor 복원 검증 실패: {e:?}"))
}

/// Owns the parsed JSON tree through product validation and error unwinding.
/// The third-party Value destructor must never recursively own this root.
#[derive(Debug)]
pub(crate) struct SnapshotJsonOwner(JsonValue);

// Retained runtime JSON must use the same iterative destruction as snapshots.
// Replacement is a move: no encoding, value change, or hash-policy change.
pub(crate) fn replace_retained_json(target: &mut JsonValue, value: JsonValue) {
    drop(SnapshotJsonOwner::from_value(std::mem::replace(target, value)));
}

pub(crate) fn clear_retained_json_option(target: &mut Option<JsonValue>) {
    if let Some(value) = target.take() { drop(SnapshotJsonOwner::from_value(value)); }
}

pub(crate) fn clear_retained_json_rows(target: &mut Vec<JsonValue>) {
    for value in target.drain(..) { drop(SnapshotJsonOwner::from_value(value)); }
}

impl SnapshotJsonOwner {
    /// Ownership only: this constructor does not admit source or saved state.
    pub(crate) fn from_value(value: JsonValue) -> Self { Self(value) }
    pub(crate) fn value_mut(&mut self) -> &mut JsonValue { &mut self.0 }
    pub(crate) fn into_parent_value(mut self) -> JsonValue { std::mem::replace(&mut self.0, JsonValue::Null) }
    pub(crate) fn compact_text(&self) -> String {
        let mut text = serialize_resource_snapshot_json(self);
        // Ordinary engine responses historically have no Bytes snapshot LF.
        if text.ends_with('\n') { text.pop(); }
        text
    }
    pub(crate) fn clone_from_value(value: &JsonValue) -> Self {
        Self::copy_from_value(value, false)
    }
    pub(crate) fn canonical_from_value(value: &JsonValue) -> Self {
        Self::copy_from_value(value, true)
    }
    fn copy_from_value(value: &JsonValue, sort_keys: bool) -> Self {
        // F5-GA[WIRE_CLONE] One root job produces one built root; each private parent job is scheduled behind exactly its child jobs and consumes only their results.
        enum Job<'a> { Value(&'a JsonValue), Array(usize), Object(Vec<&'a str>) }
        let mut jobs = vec![Job::Value(value)];
        let mut built = Vec::<SnapshotJsonOwner>::new();
        while let Some(job) = jobs.pop() {
            match job {
                Job::Value(JsonValue::Array(items)) => {
                    jobs.push(Job::Array(items.len()));
                    jobs.extend(items.iter().rev().map(Job::Value));
                }
                Job::Value(JsonValue::Object(items)) => {
                    let mut entries: Vec<_> = items.iter().collect();
                    if sort_keys { entries.sort_by(|(a, _), (b, _)| a.cmp(b)); }
                    jobs.push(Job::Object(entries.iter().map(|(key, _)| key.as_str()).collect()));
                    jobs.extend(entries.into_iter().rev().map(|(_, value)| Job::Value(value)));
                }
                Job::Value(value) => built.push(Self(value.clone())),
                Job::Array(count) => {
                    let items = built.split_off(built.len() - count);
                    built.push(Self(JsonValue::Array(items.into_iter().map(Self::into_parent_value).collect())));
                }
                Job::Object(keys) => {
                    let items = built.split_off(built.len() - keys.len());
                    built.push(Self(JsonValue::Object(keys.into_iter().map(str::to_owned)
                        .zip(items.into_iter().map(Self::into_parent_value)).collect())));
                }
            }
        }
        built.pop().expect("cloned JSON root")
    }
    pub(crate) fn equals_json(&self, other: &JsonValue) -> bool {
        let mut pending = vec![(&self.0, other)];
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (JsonValue::Array(left), JsonValue::Array(right)) => {
                    if left.len() != right.len() { return false; }
                    pending.extend(left.iter().zip(right));
                }
                (JsonValue::Object(left), JsonValue::Object(right)) => {
                    if !left.keys().eq(right.keys()) { return false; }
                    pending.extend(left.values().zip(right.values()));
                }
                (JsonValue::Array(_) | JsonValue::Object(_), _) | (_, JsonValue::Array(_) | JsonValue::Object(_)) => return false,
                _ if left != right => return false,
                _ => {},
            }
        }
        true
    }
}
impl Clone for SnapshotJsonOwner {
    fn clone(&self) -> Self { Self::clone_from_value(self) }
}
impl std::ops::Deref for SnapshotJsonOwner {
    type Target = JsonValue;
    fn deref(&self) -> &JsonValue { &self.0 }
}
impl PartialEq for SnapshotJsonOwner { fn eq(&self, other: &Self) -> bool { self.equals_json(&other.0) } }
impl PartialEq<JsonValue> for SnapshotJsonOwner { fn eq(&self, other: &JsonValue) -> bool { self.equals_json(other) } }
impl Drop for SnapshotJsonOwner {
    fn drop(&mut self) {
        let mut pending = vec![std::mem::replace(&mut self.0, JsonValue::Null)];
        while let Some(mut value) = pending.pop() {
            match &mut value {
                JsonValue::Array(items) => pending.extend(std::mem::take(items)),
                JsonValue::Object(items) => pending.extend(std::mem::take(items).into_iter().map(|(_, value)| value)),
                _ => {},
            }
        }
    }
}

pub(crate) fn parse_snapshot_json(raw: &str) -> Result<SnapshotJsonOwner, String> {
    parse_snapshot_json_with_catalog(raw,None)
}

/// A source-owned catalog is supplied by the caller, never decoded from JSON.
/// This only validates transport; resource/source admission remains separate.
pub(crate) fn parse_snapshot_json_with_catalog(raw: &str,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>) -> Result<SnapshotJsonOwner, String> {
    parse_snapshot_json_with_value_context(raw,&[],None,catalog)
}
pub(crate) fn parse_snapshot_json_with_value_context(raw:&str,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
) -> Result<SnapshotJsonOwner,String> {
    parse_bounded_json_with_value_context(raw,contracts,references,catalog,2_000_000)
}

/// Native replies are transport envelopes, not snapshot_v2 documents. Keep
/// the transport's existing 8 MiB ceiling separate from the unchanged 2 MB
/// snapshot parser. The exact same duplicate-key/carrier/catalog validation
/// and heap parser still run; this does not admit an inner stored artifact.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn parse_native_source_reply_json(raw:&str,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
) -> Result<SnapshotJsonOwner,String> {
    if raw.len()>8*1024*1024 {return Err("E_NATIVE_SOURCE_RESPONSE_LIMIT".into());}
    parse_bounded_json_with_value_context(raw,&[],None,catalog,8*1024*1024)
}

fn parse_bounded_json_with_value_context(raw:&str,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
    byte_limit:usize,
) -> Result<SnapshotJsonOwner,String> {
    // F5-GA[WIRE_JSON_STATE] A pending value either completes a scalar (then continues) or pushes a container before the frame match. Object child jobs are requested only after storing their key; close only pops the still-present matched frame and completes it for the next loop.
    // RD-694: structural depth has no authority. Heap frames/owned JSON roots
    // keep parsing and error cleanup iterative within the existing byte gate.
    if raw.len() > byte_limit { return Err("저장 스냅샷 크기 상한 초과".into()); }
    // Parsing all carrier leaves is itself one staging invocation. Do not
    // reset rational work for each JSON object before world admission.
    let mut carrier_budget=saved_resource_value::ValueDecodeBudget::new();
    enum Frame {
        Array { context: bool, values: Vec<SnapshotJsonOwner>, state: u8 },
        Object { start: usize, context: bool, values: BTreeMap<String, SnapshotJsonOwner>, keys: Vec<String>, key: Option<String>, state: u8 },
    }
    fn whitespace(bytes: &[u8], pos: &mut usize) {
        while matches!(bytes.get(*pos), Some(b' ' | b'\n' | b'\r' | b'\t')) { *pos += 1; }
    }
    fn located(raw: &str, pos: usize, message: &str) -> String {
        // F5-GA[WIRE_JSON_BOUNDS] Diagnostic prefixes clamp to byte length. scalar's start has a successful bytes.get before either suffix slice; direct pos reads are guarded by get/EOF branches. A completed Object is constructed as Object before as_object().unwrap.
        let prefix = &raw.as_bytes()[..pos.min(raw.len())];
        let line = 1 + prefix.iter().filter(|b| **b == b'\n').count();
        let column = prefix.iter().rposition(|b| *b == b'\n').map_or(prefix.len(), |p| prefix.len() - p - 1);
        format!("{message} at line {line} column {column}")
    }
    fn scalar(raw: &str, pos: &mut usize) -> Result<JsonValue, String> {
        let start = *pos;
        let bytes = raw.as_bytes();
        match bytes.get(start).copied() {
            Some(b'"') => {
                *pos += 1;
                while let Some(byte) = bytes.get(*pos) {
                    *pos += 1;
                    if *byte == b'"' { break; }
                    if *byte == b'\\' && *pos < bytes.len() { *pos += 1; }
                }
            }
            Some(b't' | b'n') => *pos = (start + 4).min(bytes.len()),
            Some(b'f') => *pos = (start + 5).min(bytes.len()),
            Some(b'-' | b'0'..=b'9') => {
                if bytes.get(*pos) == Some(&b'-') { *pos += 1; }
                while matches!(bytes.get(*pos), Some(b'0'..=b'9')) { *pos += 1; }
                if bytes.get(*pos) == Some(&b'.') {
                    *pos += 1;
                    while matches!(bytes.get(*pos), Some(b'0'..=b'9')) { *pos += 1; }
                }
                if matches!(bytes.get(*pos), Some(b'e' | b'E')) {
                    *pos += 1;
                    if matches!(bytes.get(*pos), Some(b'+' | b'-')) { *pos += 1; }
                    while matches!(bytes.get(*pos), Some(b'0'..=b'9')) { *pos += 1; }
                }
            }
            Some(_) => return Err(located(raw, start + 1, "expected value")),
            None => return Err(located(raw, start, "EOF while parsing a value")),
        }
        // Only scalar tokens reach serde: its escape and number rules remain
        // authoritative, but it never recursively parses a saved container.
        let mut decoder = serde_json::Deserializer::from_slice(&bytes[start..]);
        <JsonValue as serde::Deserialize>::deserialize(&mut decoder).map_err(|error| {
            let message = error.to_string();
            let message = message.rsplit_once(" at line ").map_or(message.as_str(), |(text, _)| text);
            let token = &bytes[start..];
            let offset = if error.line() == 1 { error.column() } else {
                token.iter().enumerate().filter(|(_, b)| **b == b'\n')
                    .nth(error.line() - 2).map_or(token.len(), |(p, _)| p + 1 + error.column())
            };
            located(raw, start + offset, message)
        })
    }
    let bytes = raw.as_bytes();
    let mut pos = 0;
    let mut frames = Vec::<Frame>::new();
    let mut need_value = Some(false);
    let mut completed = None::<SnapshotJsonOwner>;
    loop {
        if let Some(value) = completed.take() {
            match frames.last_mut() {
                Some(Frame::Array { values, state, .. }) => { values.push(value); *state = 1; }
                Some(Frame::Object { values, keys, key, state, .. }) => {
                    let key = key.take().expect("object child has a key");
                    keys.push(key.clone()); values.insert(key, value); *state = 1;
                }
                None => {
                    whitespace(bytes, &mut pos);
                    if matches!(value.get("type").and_then(JsonValue::as_str),Some("tuple_v1" | "rational_v1")) {
                        let exact = serde_json::to_string(&*value).map_err(|e| e.to_string())?;
                        if raw != exact { return Err(located(raw, pos, "위치형 저장 앞뒤 공백은 허용하지 않습니다")); }
                    }
                    if pos != bytes.len() { return Err(located(raw, pos + 1, "trailing characters")); }
                    return Ok(value);
                }
            }
        }
        if let Some(context) = need_value.take() {
            whitespace(bytes, &mut pos);
            match bytes.get(pos) {
                Some(b'[' | b'{') => {
                    let frame = if bytes[pos] == b'[' {
                        Frame::Array { context, values: Vec::new(), state: 0 }
                    } else { Frame::Object { start: pos, context, values: BTreeMap::new(), keys: Vec::new(), key: None, state: 0 } };
                    frames.push(frame); pos += 1;
                }
                _ => { completed = Some(SnapshotJsonOwner(scalar(raw, &mut pos)?)); continue; }
            }
        }
        whitespace(bytes, &mut pos);
        let mut close = false;
        match frames.last_mut().expect("pending container") {
            Frame::Array { context, state, .. } => {
                if bytes.get(pos) == Some(&b']') {
                    if *state == 2 { return Err(located(raw, pos + 1, "trailing comma")); }
                    close = true;
                } else if *state == 1 {
                    match bytes.get(pos) {
                        Some(b',') => { pos += 1; *state = 2; }
                        Some(_) => return Err(located(raw, pos + 1, "expected `,` or `]`")),
                        None => return Err(located(raw, pos, "EOF while parsing a list")),
                    }
                } else {
                    if pos == bytes.len() { return Err(located(raw, pos, if *state == 0 { "EOF while parsing a list" } else { "EOF while parsing a value" })); }
                    need_value = Some(*context);
                }
            }
            Frame::Object { context, values, key, state, .. } => {
                if bytes.get(pos) == Some(&b'}') {
                    if *state == 2 { return Err(located(raw, pos + 1, "trailing comma")); }
                    close = true;
                } else if *state == 1 {
                    match bytes.get(pos) {
                        Some(b',') => { pos += 1; *state = 2; }
                        Some(_) => return Err(located(raw, pos + 1, "expected `,` or `}`")),
                        None => return Err(located(raw, pos, "EOF while parsing an object")),
                    }
                } else {
                    if pos == bytes.len() { return Err(located(raw, pos, if *state == 0 { "EOF while parsing an object" } else { "EOF while parsing a value" })); }
                    if bytes[pos] != b'"' { return Err(located(raw, pos + 1, "key must be a string")); }
                    let JsonValue::String(name) = scalar(raw, &mut pos)? else { unreachable!() };
                    if values.contains_key(&name) {
                        // serde runs end_map even after the visitor rejects a
                        // duplicate, then attaches its current read position.
                        whitespace(bytes, &mut pos);
                        if bytes.get(pos) == Some(&b'}') { pos += 1; }
                        return Err(located(raw, pos, "저장 자원 키 중복"));
                    }
                    whitespace(bytes, &mut pos);
                    match bytes.get(pos) {
                        Some(b':') => pos += 1,
                        Some(_) => return Err(located(raw, pos + 1, "expected `:`")),
                        None => return Err(located(raw, pos, "EOF while parsing an object")),
                    }
                    need_value = Some(*context || name == "value_det");
                    *key = Some(name);
                }
            }
        }
        if close {
            let frame = frames.pop().unwrap();
            let value = match frame {
                Frame::Array { values, .. } => SnapshotJsonOwner(JsonValue::Array(values.into_iter().map(SnapshotJsonOwner::into_parent_value).collect())),
                Frame::Object { start, context, values, keys, .. } => {
                    let value = SnapshotJsonOwner(JsonValue::Object(values.into_iter().map(|(key, value)| (key, value.into_parent_value())).collect()));
                    if matches!(value.get("type").and_then(JsonValue::as_str),Some("tuple_v1" | "rational_v1")) {
                        if keys.iter().map(String::as_str).ne(tuple_value::FIELDS) {
                            return Err(located(raw, pos + 1, "위치형 저장 필드 순서가 다릅니다"));
                        }
                        let candidate = tuple_value::candidate_with_budget(&value,&mut carrier_budget,contracts,references,catalog).map_err(|error| located(raw, pos + 1, &error))?;
                        let canonical = serde_json::to_string(&resource_value_to_detjson(&candidate)).map_err(|e| e.to_string())?;
                        if raw.get(start..=pos) != Some(canonical.as_str()) {
                            return Err(located(raw, pos + 1, "위치형 저장은 정본 ASCII JSON이어야 합니다"));
                        }
                    }
                    if value.get("type").and_then(JsonValue::as_str) == Some("bytes_v1") {
                        if keys.iter().map(String::as_str).ne(bytes_value::BYTES_FIELDS.iter().copied()) {
                            return Err(located(raw, pos + 1, "E_BYTES_SNAPSHOT: 바이트열 필드 순서가 다릅니다"));
                        }
                        bytes_value::parse_bytes_v1(value.as_object().unwrap()).map_err(|error| located(raw, pos + 1, &error))?;
                    }
                    if context && value.get("type").and_then(JsonValue::as_str) == Some("tensor_v1") {
                        tensor_frame::validate_transport_keys(keys.iter().map(String::as_str)).map_err(|_| located(raw, pos + 1, "Tensor 키 중복 또는 순서 오류"))?;
                        tensor_frame::transport_from_value(&value).map_err(|error| located(raw, pos + 1, &format!("Tensor 저장 검증 실패: {error:?}")))?;
                    }
                    value
                }
            };
            pos += 1; completed = Some(value);
        }
    }
}

// Frozen pre-frame parser is a bounded test oracle, never a product fallback.
#[cfg(test)]
fn parse_snapshot_json_recursive_oracle(raw: &str) -> Result<SnapshotJsonOwner, String> {
    use serde::de::{DeserializeSeed, Visitor, MapAccess, SeqAccess, Error};
    struct Seed(bool);
    impl<'de> DeserializeSeed<'de> for Seed {
        type Value = SnapshotJsonOwner;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_any(self)
        }
    }
    impl<'de> Visitor<'de> for Seed {
        type Value = SnapshotJsonOwner;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("저장 스냅샷") }
        fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(json!(v))) }
        fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(json!(v))) }
        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(json!(v))) }
        fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(json!(v))) }
        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(json!(v))) }
        fn visit_string<E: Error>(self, v: String) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(JsonValue::String(v))) }
        fn visit_unit<E: Error>(self) -> Result<Self::Value,E> { Ok(SnapshotJsonOwner(JsonValue::Null)) }
        fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value,S::Error> {
            let mut out = Vec::new();
            while let Some(v) = seq.next_element_seed(Seed(self.0))? { out.push(v); }
            Ok(SnapshotJsonOwner(JsonValue::Array(out.into_iter().map(SnapshotJsonOwner::into_parent_value).collect())))
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value,M::Error> {
            let mut keys = Vec::new();
            let mut obj = BTreeMap::new();
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
            let value = SnapshotJsonOwner(JsonValue::Object(obj.into_iter().map(|(key, value)| (key, value.into_parent_value())).collect()));
            if value.get("type").and_then(JsonValue::as_str) == Some("bytes_v1") {
                if keys.iter().map(String::as_str).ne(bytes_value::BYTES_FIELDS.iter().copied()) {
                    return Err(M::Error::custom("E_BYTES_SNAPSHOT: 바이트열 필드 순서가 다릅니다"));
                }
                bytes_value::parse_bytes_v1(value.as_object().ok_or_else(||M::Error::custom("바이트열 object 필요"))?)
                    .map_err(M::Error::custom)?;
            }
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
    fn native_reply_envelope_keeps_snapshot_and_transport_limits_separate() {
        // A stored JSON string is still under snapshot_v2's 2 MB ceiling;
        // escaped JSON in its transport envelope can exceed 2 MB. This test
        // grants no artifact, world or execution admission.
        let inner=serde_json::json!({"data":"\"".repeat(700_000)}).to_string();
        assert!(inner.len()<2_000_000);
        assert!(parse_snapshot_json(&inner).is_ok());
        let reply=serde_json::json!({"saved_artifact_json_utf8":inner,"kind":"native_general_world"}).to_string();
        assert!(reply.len()>2_000_000 && reply.len()<8*1024*1024);
        assert!(parse_snapshot_json(&reply).is_err(),"snapshot_v2 limit must not rise");
        let parsed=parse_native_source_reply_json(&reply,None).unwrap();
        assert_eq!(parsed["kind"].as_str(),Some("native_general_world"));
        assert_eq!(parsed["saved_artifact_json_utf8"].as_str(),Some(inner.as_str()));
        assert!(parse_native_source_reply_json("{\"x\":1,\"x\":2}",None).is_err());
        assert!(parse_native_source_reply_json("[1-2]",None).is_err());
        let too_large=" ".repeat(8*1024*1024+1);
        assert_eq!(parse_native_source_reply_json(&too_large,None).unwrap_err(),"E_NATIVE_SOURCE_RESPONSE_LIMIT");
    }

    #[test]
    fn frame_json_parser_preserves_bounded_oracle_results_and_diagnostics() {
        let seeds = ["null", "true", "false", "12.5", "-0.08", "\"한글\\n글\"", "[]", "{}",
            "1e-20", "-1.25E+2", "18446744073709551615", "0.000001", "\"\\uD83D\\uDE00\"",
            "[1-2]", "[1.2.3]", "[01]", "[truex]", "[n한글]", "[\"\\uD800\"]", "[1e999]",
            "[1,2,{\"값\":[false,null]}]", "{\"a\":1,\"b\":[2,3]}",
            "{\"a\":1,\"a\":2}", "{\"type\":\"bytes_v1\"}", "{\"value_det\":{\"type\":\"tensor_v1\"}}"];
        let compare = |raw: &str| {
            let actual = parse_snapshot_json(raw);
            let expected = parse_snapshot_json_recursive_oracle(raw);
            match (actual, expected) {
                (Ok(a), Ok(b)) => assert_eq!(a, b, "raw={raw:?}"),
                (Err(a), Err(b)) => assert_eq!(a, b, "raw={raw:?}"),
                (a, b) => panic!("admission mismatch raw={raw:?}: actual={a:?}, expected={b:?}"),
            }
        };
        for seed in seeds {
            compare(seed);
            compare(&format!(" \n {seed} \n "));
            compare(&format!("{seed} false"));
            for (pos, _) in seed.char_indices() {
                compare(&seed[..pos]);
                for replacement in ["", ",", ":", "]", "}", "x", "\n"] {
                    let next = pos + seed[pos..].chars().next().unwrap().len_utf8();
                    compare(&format!("{}{}{}", &seed[..pos], replacement, &seed[next..]));
                }
            }
        }
        // The old third-party recursion ceiling is deliberately superseded by
        // RD-694. Differential syntax checks above retain shallow byte parity.
        for depth in [80, 126, 127] {
            compare(&format!("{}0{}", "[".repeat(depth), "]".repeat(depth)));
        }
    }

    #[test]
    fn frame_json_parser_handles_deep_values_and_rejection_on_small_stack() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let valid = format!("{}0{}", "[".repeat(20_000), "]".repeat(20_000));
            drop(parse_snapshot_json(&valid).unwrap());
            let child = format!("{}0{}", "[".repeat(20_000), "]".repeat(20_000));
            for raw in [format!("[{child},invalid]"), format!("{{\"a\":{child},\"a\":0}}"), format!("{valid} false")] {
                assert!(parse_snapshot_json(&raw).is_err());
            }
        }).unwrap().join().unwrap();
    }

    #[test]
    fn current_raw_snapshot_preserves_bytes_order_and_other_type_guards() {
        let bytes=ResourceValue::Bytes(ddonirang_core::bytes::ByteSequence::new(vec![0,255]));
        let leaf=resource_value_to_detjson(&bytes).to_string();
        let raw=format!("{{\"resources\":{{\"value_det\":{{\"자료\":{leaf}}}}}}}");
        let parsed=parse_snapshot_json(&raw).unwrap();
        assert_eq!(resource_value_from_detjson(&parsed["resources"]["value_det"]["자료"]).unwrap(),bytes);
        let fields=serde_json::from_str::<JsonValue>(&leaf).unwrap();
        let reversed=fields.as_object().unwrap().iter().rev().map(|(k,v)|
            format!("{}:{v}",serde_json::to_string(k).unwrap())).collect::<Vec<_>>().join(",");
        for bad in [raw.replacen(&leaf,&format!("{{{reversed}}}"),1),
            raw.replacen(&leaf,&leaf.replacen("{","{\"version\":1,",1),1),
            raw.replacen("\"version\":1","\"version\":2",1),
            raw.replacen("\"payload_byte_length\":2","\"payload_byte_length\":3",1)] {
            assert!(parse_snapshot_json(&bad).is_err(),"accepted {bad}");
        }
        let serialized=serialize_resource_snapshot_json(&parsed);
        assert!(serialized.ends_with('\n'));
        assert_eq!(parse_snapshot_json(&serialized).unwrap(),parsed);
        assert_eq!(serialize_resource_snapshot_json(&json!({"ordinary":1})),"{\"ordinary\":1}");
        // Current snapshot duplicate policy is stricter than old non-Bytes JSON;
        // the Bytes owner must not weaken it during integration.
        assert!(parse_snapshot_json("{\"ordinary\":1,\"ordinary\":2}").is_err());
    }

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

// RD-660 six-key reference leaf. Parsing never grants LIVE custody.
pub(crate) fn reference_to_detjson(value: &ddonirang_core::platform::IdentityReferenceValueV1) -> JsonValue {
    json!({
        "type": "identity_reference_v1",
        "profile_version": value.profile_version,
        "target_type_identity_hex": hex::encode(&value.target_type_identity),
        "owner_space_identity_hex": hex::encode(&value.owner_space_identity),
        "target_identity_hex": hex::encode(&value.target_identity),
        "generation_u64": value.generation.to_string(),
    })
}

pub(crate) fn reference_from_detjson(
    payload: &JsonValue,
) -> Result<ddonirang_core::platform::IdentityReferenceValueV1, String> {
    let bad = || "SUBJECT_REFERENCE_TUPLE_MALFORMED: 참조 저장값이 올바르지 않습니다".to_string();
    let obj = payload.as_object().ok_or_else(bad)?;
    let keys = [
        "type",
        "profile_version",
        "target_type_identity_hex",
        "owner_space_identity_hex",
        "target_identity_hex",
        "generation_u64",
    ];
    if obj.len() != keys.len()
        || keys.iter().any(|key| !obj.contains_key(*key))
        || obj["type"].as_str() != Some("identity_reference_v1")
        || obj["profile_version"].as_u64() != Some(1)
    {
        return Err(bad());
    }
    let identity = |key: &str| -> Result<Vec<u8>, String> {
        let text = obj[key].as_str().ok_or_else(bad)?;
        let max=if key=="target_type_identity_hex"{4096}else{32};
        // RD-660/668/694: fixed identity lengths must reject before decode,
        // not after allocating a temporary identity buffer.
        if text.is_empty() || text.len()>max*2
            || (key != "target_type_identity_hex" && text.len() != 64)
            || text.len() % 2 != 0
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(bad());
        }
        hex::decode(text).map_err(|_|bad())
    };
    let raw_generation = obj["generation_u64"].as_str().ok_or_else(bad)?;
    let generation = raw_generation.parse::<u64>().map_err(|_| bad())?;
    if generation.to_string() != raw_generation {
        return Err(bad());
    }
    Ok(ddonirang_core::platform::IdentityReferenceValueV1 {
        profile_version: 1,
        target_type_identity: identity("target_type_identity_hex")?,
        owner_space_identity: identity("owner_space_identity_hex")?,
        target_identity: identity("target_identity_hex")?,
        generation,
    })
}

fn registered_atom_to_detjson(value: &ddonirang_core::platform::RegisteredAtomValueV1) -> JsonValue {
    json!({
        "type": "registered_atom_v1",
        "profile_version": value.profile_version,
        "owner_registry_identity_hex": hex::encode(&value.owner_registry_identity),
        "registered_atom_identity_hex": hex::encode(&value.registered_atom_identity),
        "contract_profile": value.contract_profile,
        "contract_version": value.contract_version,
        "contract_snapshot_identity_hex": hex::encode(&value.contract_snapshot_identity),
        "contract_sha256": hex::encode(value.contract_sha256),
    })
}

pub(crate) fn registered_atom_from_detjson(
    payload: &JsonValue,
) -> Result<RegisteredAtomValueV1, String> {
    let bad =
        || "REGISTERED_ATOM_SAVED_VALUE_MALFORMED: 등록씨 저장값이 올바르지 않습니다".to_string();
    let obj = payload.as_object().ok_or_else(bad)?;
    let keys = [
        "type",
        "profile_version",
        "owner_registry_identity_hex",
        "registered_atom_identity_hex",
        "contract_profile",
        "contract_version",
        "contract_snapshot_identity_hex",
        "contract_sha256",
    ];
    if obj.len() != keys.len()
        || keys.iter().any(|key| !obj.contains_key(*key))
        || obj["type"].as_str() != Some("registered_atom_v1")
        || obj["profile_version"].as_u64() != Some(1)
        || obj["contract_profile"].as_str() != Some("ddn.registered-atom-contract.v1")
        || obj["contract_version"].as_u64() != Some(1)
    {
        return Err(bad());
    }
    let identity = |key: &str| -> Result<Vec<u8>, String> {
        let text = obj[key].as_str().ok_or_else(bad)?;
        let fixed_digest = matches!(key, "contract_sha256" | "contract_snapshot_identity_hex");
        if text.is_empty()
            || text.len() % 2 != 0
            || (fixed_digest && text.len() != 64)
            || (!fixed_digest && text.len() > 8192)
            || !text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(bad());
        }
        hex::decode(text).map_err(|_| bad())
    };
    let contract_sha = identity("contract_sha256")?;
    let contract_sha256: [u8; 32] = contract_sha.try_into().map_err(|_| bad())?;
    let contract_snapshot_identity = identity("contract_snapshot_identity_hex")?;
    Ok(RegisteredAtomValueV1 {
        profile_version: 1,
        owner_registry_identity: identity("owner_registry_identity_hex")?,
        registered_atom_identity: identity("registered_atom_identity_hex")?,
        contract_profile: "ddn.registered-atom-contract.v1".to_string(),
        contract_version: 1,
        contract_snapshot_identity,
        contract_sha256,
    })
}

pub fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    // F5-GA[WIRE_PROJECT_JOBS] Map jobs produce exactly two children per row; Record jobs produce types.len+fields.len children. Parent jobs run after those children and consume the same immutable counts, leaving one root result.
    enum Job<'a> {
        Value(&'a ResourceValue), Type(&'a RecordFieldType),
        Collection(&'static str, usize), Map(usize),
        Record(&'a BTreeMap<String, RecordFieldType>, &'a BTreeMap<String, ResourceValue>),
        Applied(&'a str, usize), TupleType(usize), RecordType(&'a BTreeMap<String, RecordFieldType>),
    }
    // A failed projection must not recursively destroy completed siblings.
    struct Built(Vec<JsonValue>);
    impl Drop for Built {
        fn drop(&mut self) {
            while let Some(mut value) = self.0.pop() {
                match &mut value {
                    JsonValue::Array(items) => self.0.extend(std::mem::take(items)),
                    JsonValue::Object(items) => self.0.extend(std::mem::take(items).into_iter().map(|(_, value)| value)),
                    _ => {},
                }
            }
        }
    }
    let object = |fields: Vec<(&str, JsonValue)>| JsonValue::Object(fields.into_iter().map(|(key, value)| (key.to_owned(), value)).collect());
    let mut jobs = vec![Job::Value(value)];
    let mut built = Built(Vec::new());
    while let Some(job) = jobs.pop() {
        match job {
            Job::Value(ResourceValue::List(items)) => {
                jobs.push(Job::Collection("list", items.len()));
                jobs.extend(items.iter().rev().map(Job::Value));
            }
            Job::Value(ResourceValue::Set(items)) => {
                jobs.push(Job::Collection("set", items.len()));
                jobs.extend(items.values().rev().map(Job::Value));
            }
            Job::Value(ResourceValue::Map(items)) => {
                jobs.push(Job::Map(items.len()));
                for entry in items.values().rev() { jobs.push(Job::Value(&entry.value)); jobs.push(Job::Value(&entry.key)); }
            }
            Job::Value(ResourceValue::Record { field_types, fields }) => {
                jobs.push(Job::Record(field_types, fields));
                jobs.extend(fields.values().rev().map(Job::Value));
                jobs.extend(field_types.values().rev().map(Job::Type));
            }
            Job::Value(value) => built.0.push(resource_value_leaf_to_detjson(value)),
            Job::Type(RecordFieldType::Tuple { items }) => {
                jobs.push(Job::TupleType(items.len()));
                jobs.extend(items.iter().rev().map(Job::Type));
            }
            Job::TupleType(count) => {
                let items = built.0.split_off(built.0.len() - count);
                built.0.push(object(vec![("kind", "Tuple".into()), ("items", JsonValue::Array(items))]));
            }
            Job::Type(RecordFieldType::Named { name }) => built.0.push(json!({"kind":"Named", "name":name})),
            // Diagnostic projection only: the saved decoder rejects this
            // output-only descriptor; it is not a saved type allocation.
            Job::Type(RecordFieldType::RegisteredAtomV1) => built.0.push(json!({"kind":"UnsupportedOutputOnly"})),
            Job::Type(RecordFieldType::ResolvedNominalV1{category,name_type_identity})=>built.0.push(json!({"kind":"ResolvedNominalV1","category":category,"name_type_identity":name_type_identity})),
            Job::Type(RecordFieldType::Applied { name, args }) => {
                jobs.push(Job::Applied(name, args.len()));
                jobs.extend(args.iter().rev().map(Job::Type));
            }
            Job::Type(RecordFieldType::Record { fields }) => {
                jobs.push(Job::RecordType(fields));
                jobs.extend(fields.values().rev().map(Job::Type));
            }
            Job::Collection(kind, count) => {
                let items = built.0.split_off(built.0.len() - count);
                built.0.push(object(vec![("type", kind.into()), ("items", JsonValue::Array(items))]));
            }
            Job::Map(count) => {
                let items = built.0.split_off(built.0.len() - count * 2);
                let mut items = items.into_iter();
                let mut rows = Vec::with_capacity(count);
                while let Some(key) = items.next() {
                    rows.push(object(vec![("key", key), ("value", items.next().expect("projected map value"))]));
                }
                built.0.push(object(vec![("type", "map".into()), ("entries", JsonValue::Array(rows))]));
            }
            Job::Record(types, fields) => {
                let items = built.0.split_off(built.0.len() - types.len() - fields.len());
                let mut items = items.into_iter();
                let identities = JsonValue::Object(types.keys().map(|name| (name.clone(), items.next().expect("projected field type"))).collect());
                let rows = fields.keys().map(|name| object(vec![("name", name.as_str().into()), ("value", items.next().expect("projected record field"))])).collect();
                built.0.push(object(vec![("type", "record".into()), ("version", 1.into()), ("type_identity", identities), ("fields", JsonValue::Array(rows))]));
            }
            Job::Applied(name, count) => {
                let args = built.0.split_off(built.0.len() - count);
                built.0.push(object(vec![("kind", "Applied".into()), ("name", name.into()), ("args", JsonValue::Array(args))]));
            }
            Job::RecordType(fields) => {
                let values = built.0.split_off(built.0.len() - fields.len());
                let fields = JsonValue::Object(fields.keys().cloned().zip(values).collect());
                built.0.push(object(vec![("kind", "Record".into()), ("fields", fields)]));
            }
        }
    }
    built.0.pop().expect("projected saved value")
}

fn resource_value_leaf_to_detjson(value: &ResourceValue) -> JsonValue {
    match value {
        ResourceValue::Tuple { .. } | ResourceValue::RationalV1(_) => {
            use sha2::{Digest, Sha256};
            let mut canonical = Vec::new(); value.encode_canon(&mut canonical);
            json!({"canonical_base64url":ddonirang_core::commit_canon::carrier_base64url(&canonical),
                "canonical_byte_length":canonical.len(),"canonical_sha256":hex::encode(Sha256::digest(&canonical)),
                "resource_value_tag":if matches!(value, ResourceValue::RationalV1(_)) {"10"} else {"0f"},
                "type":if matches!(value, ResourceValue::RationalV1(_)) {"rational_v1"} else {"tuple_v1"},"version":1})
        }
        ResourceValue::IdentityReference(value) => reference_to_detjson(value),
        ResourceValue::NominalValueV1(_) | ResourceValue::SubjectHandleV1(_) | ResourceValue::SubjectPayloadV1(_) => {
            let mut canonical=Vec::new(); value.encode_canon(&mut canonical);
            json!({"type":"vc04_canonical_carrier", "canonical_base64url":ddonirang_core::commit_canon::carrier_base64url(&canonical),
                "resource_value_tag":format!("{:02x}",canonical[0]), "version":1})
        }
        ResourceValue::RegisteredAtom(value) => registered_atom_to_detjson(value),
        ResourceValue::Bytes(value) => json!({
            "frame_base64url": value.storage_base64url(),
            "frame_byte_length": value.len() + 20,
            "frame_sha256": hex::encode(value.storage_sha256()),
            "payload_byte_length": value.len(),
            "payload_sha256": hex::encode(value.content_sha256()),
            "resource_value_tag": "0c", "type": "bytes_v1", "version": 1,
        }),
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
        ResourceValue::List(_) | ResourceValue::Set(_) | ResourceValue::Map(_) | ResourceValue::Record { .. } => unreachable!("structural projection is owned by heap jobs"),
    }
}

pub fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    resource_value_from_detjson_with_contracts(payload, &[])
}

pub(crate) fn resource_value_from_detjson_with_contracts(payload: &JsonValue, contracts: &[crate::registered_atom_contract::RecordedAtomContract]) -> Result<ResourceValue, String> {
    resource_value_from_detjson_with_custody(payload,contracts,None)
}

pub(crate) fn resource_value_from_detjson_with_custody(payload:&JsonValue,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>)->Result<ResourceValue,String>{
    let value = resource_value_candidate_from_detjson(payload, contracts, references)?;
    if tuple_value::requires_source_type(&value) { return Err("E_TUPLE_PERSISTENCE_UNBOUND: 복원 원문·위치씨 공급이 필요합니다".into()); }
    Ok(value)
}

fn resource_value_candidate_from_detjson(payload:&JsonValue,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>)->Result<ResourceValue,String>{
    resource_value_candidate_from_detjson_with_budget(payload, contracts, references, &mut saved_resource_value::ValueDecodeBudget::new())
}

fn resource_value_candidate_from_detjson_with_budget(payload:&JsonValue,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,budget:&mut saved_resource_value::ValueDecodeBudget)->Result<ResourceValue,String>{
    resource_value_candidate_from_detjson_with_catalog(payload,contracts,references,budget,None)
}
fn resource_value_candidate_from_detjson_with_catalog(payload:&JsonValue,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,budget:&mut saved_resource_value::ValueDecodeBudget,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<ResourceValue,String>{
    crate::registered_atom_contract::verify_contract_set(contracts).map_err(|e|e.code().to_owned())?;
    verify_identity_tree(payload, contracts, references,0)?;
    resource_value_from_admitted_detjson_with_catalog(payload,budget,contracts,references,catalog)
}

// Validate every leaf before Set/Map canonical keys can coalesce values.
fn verify_identity_tree(payload: &JsonValue, contracts: &[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>, _depth: usize) -> Result<(), String> {
    // RD-694: preserve first-invalid-leaf order without a host-depth cutoff.
    let mut pending = vec![payload];
    while let Some(payload) = pending.pop() {
        match payload {
        JsonValue::Object(obj) => {
            if obj.get("type").and_then(JsonValue::as_str) == Some("registered_atom_v1") {
                let atom = registered_atom_from_detjson(payload)?;
                crate::registered_atom_contract::verify_recorded_atom(&atom,contracts).map_err(|e|e.code().to_owned())?;
            } else if obj.get("type").and_then(JsonValue::as_str)==Some("identity_reference_v1") {
                let value=reference_from_detjson(payload)?;
                references.ok_or_else(||"SUBJECT_REFERENCE_CUSTODY_REQUIRED: 저장된 참조 정체가 필요합니다".to_owned())?
                    .verify(&value).map_err(|_|"SUBJECT_REFERENCE_CUSTODY_INVALID: 저장된 참조 정체가 맞지 않습니다".to_owned())?;
            } else { pending.extend(obj.values().rev()); }
        }
        JsonValue::Array(items) => { pending.extend(items.iter().rev()); }
        _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
fn resource_value_from_admitted_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    resource_value_from_admitted_detjson_with_budget(payload, &mut saved_resource_value::ValueDecodeBudget::new())
}

fn resource_value_from_admitted_detjson_with_budget(payload: &JsonValue, budget: &mut saved_resource_value::ValueDecodeBudget) -> Result<ResourceValue, String> {
    resource_value_from_admitted_detjson_with_catalog(payload,budget,&[],None,None)
}
fn resource_value_from_admitted_detjson_with_catalog(payload:&JsonValue,budget:&mut saved_resource_value::ValueDecodeBudget,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<ResourceValue,String>{
    // F5-GA[WIRE_RESTORE_JOBS] Each admitted child produces one value or returns Err. MapRow/MapValue schedule exactly paired values; every RecordRow validates a nonempty String name before its parent consumes all rows. Immutable jobs leave exactly one root on success.
    enum Job<'a> {
        Value(&'a JsonValue), List(usize), Set(usize), Map(usize),
        MapRow(&'a JsonValue), MapValue(&'a serde_json::Map<String, JsonValue>),
        RecordRow(&'a [JsonValue], usize, Option<&'a str>),
        Record(BTreeMap<String, RecordFieldType>, &'a [JsonValue]),
    }
    let record_bad = || "E_PRODUCT_RECORD_WIRE: 올바르지 않은 묶음 저장값입니다".to_string();
    let mut jobs = vec![Job::Value(payload)];
    // ResourceValue owns stack-safe Drop, including partial staging errors.
    let mut built: Vec<ResourceValue> = Vec::new();
    while let Some(job) = jobs.pop() {
        match job {
            Job::Value(value) => {
                let obj = value.as_object().ok_or_else(|| "value_det 형식 오류(object 아님)".to_string())?;
                match obj.get("type").and_then(JsonValue::as_str).unwrap_or_default() {
                    "vc04_canonical_carrier"=>{
                        let catalog=catalog.ok_or("E_VC04_REGISTERED_CONTEXT: 실제 원문 등록 catalog가 필요합니다")?;
                        if obj.len()!=4||obj.get("version").and_then(JsonValue::as_u64)!=Some(1){return Err("E_VC04_CARRIER_FIELDS".into());}
                        let encoded=obj.get("canonical_base64url").and_then(JsonValue::as_str).ok_or("E_VC04_CARRIER_BYTES")?;
                        if encoded.len()>2_666_668{return Err("E_VC04_CARRIER_LIMIT".into());}
                        budget.charge(encoded.len().checked_mul(3).ok_or("E_VC04_CARRIER_LIMIT")?).map_err(|_|"E_VC04_CARRIER_LIMIT")?;
                        let raw=tensor_frame::decode_base64url(encoded).map_err(|_|"E_VC04_CARRIER_BYTES")?;
                        let expected=match obj.get("resource_value_tag").and_then(JsonValue::as_str){Some("11")=>0x11,Some("12")=>0x12,Some("13")=>0x13,_=>return Err("E_VC04_CARRIER_TAG".into())};
                        if raw.first()!=Some(&expected){return Err("E_VC04_CARRIER_TAG".into());}
                        // JSON transports bytes only; the source-owned catalog
                        // and shared binary decoder own typed admission.
                        built.push(saved_resource_value::decode_saved_resource_value_with_catalog(&raw,budget,contracts,references,Some(catalog),false)
                            .map_err(|error|format!("E_VC04_CARRIER_ADMISSION: {error:?}"))?);
                    }
                    kind @ ("list" | "set") => {
                        let items = obj.get("items").and_then(JsonValue::as_array)
                            .ok_or_else(|| format!("{kind} items 형식 오류: 목록이 필요합니다"))?;
                        jobs.push(if kind == "list" { Job::List(items.len()) } else { Job::Set(items.len()) });
                        jobs.extend(items.iter().rev().map(Job::Value));
                    }
                    "map" => {
                        let rows = obj.get("entries").and_then(JsonValue::as_array)
                            .ok_or_else(|| "map entries 형식 오류: 목록이 필요합니다".to_string())?;
                        jobs.push(Job::Map(rows.len()));
                        jobs.extend(rows.iter().rev().map(Job::MapRow));
                    }
                    "record" => {
                        let (types, rows) = parse_record_header(obj)?;
                        jobs.push(Job::Record(types, rows));
                        jobs.push(Job::RecordRow(rows, 0, None));
                    }
                    _ => built.push(resource_value_leaf_from_admitted_detjson(value,budget,contracts,references,catalog)?),
                }
            }
            Job::List(count) => {
                let items = built.split_off(built.len() - count);
                built.push(ResourceValue::List(items));
            }
            Job::Set(count) => {
                let items = built.split_off(built.len() - count);
                built.push(ResourceValue::set_from_values(items));
            }
            Job::Map(count) => {
                let items = built.split_off(built.len() - count * 2);
                let mut items = items.into_iter();
                let mut entries = Vec::with_capacity(count);
                while let Some(key) = items.next() { entries.push(ResourceMapEntry { key, value: items.next().expect("restored map value") }); }
                built.push(ResourceValue::map_from_entries(entries));
            }
            Job::MapRow(row) => {
                let row = row.as_object().ok_or_else(|| "map entry 형식 오류".to_string())?;
                let key = row.get("key").ok_or_else(|| "map entry key 누락".to_string())?;
                // Preserve key admission before a missing/invalid value and
                // before any later row's structural diagnostic.
                jobs.push(Job::MapValue(row)); jobs.push(Job::Value(key));
            }
            Job::MapValue(row) => jobs.push(Job::Value(row.get("value").ok_or_else(|| "map entry value 누락".to_string())?)),
            Job::RecordRow(rows, index, previous) => {
                if let Some(row) = rows.get(index) {
                    let row = row.as_object().ok_or_else(record_bad)?;
                    if row.len() != 2 || !row.contains_key("name") || !row.contains_key("value") { return Err(record_bad()); }
                    let name = row["name"].as_str().filter(|name| !name.is_empty()).ok_or_else(record_bad)?;
                    if previous.is_some_and(|previous| previous >= name) { return Err(record_bad()); }
                    validate_record_wire_value(&row["value"])?;
                    jobs.push(Job::RecordRow(rows, index + 1, Some(name)));
                    jobs.push(Job::Value(&row["value"]));
                }
            }
            Job::Record(field_types, rows) => {
                let values = built.split_off(built.len() - rows.len());
                let fields: BTreeMap<_, _> = rows.iter().map(|row| row["name"].as_str().expect("validated record name").to_owned()).zip(values).collect();
                if !field_types.keys().eq(fields.keys()) { return Err(record_bad()); }
                let value = ResourceValue::Record { field_types, fields };
                let runtime = crate::ddn_runtime::value_from_resource_value(&value);
                ddonirang_lang::runtime::validate_record_value(&runtime).map_err(|_| record_bad())?;
                built.push(value);
            }
        }
    }
    Ok(built.pop().expect("restored saved value"))
}

fn resource_value_leaf_from_admitted_detjson(payload: &JsonValue, budget: &mut saved_resource_value::ValueDecodeBudget,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>) -> Result<ResourceValue, String> {
    let JsonValue::Object(obj) = payload else {
        return Err("value_det 형식 오류(object 아님)".to_string());
    };
    let kind = obj.get("type").and_then(JsonValue::as_str).unwrap_or_default();
    match kind {
        "registered_atom_v1" => registered_atom_from_detjson(payload).map(|value|ResourceValue::RegisteredAtom(value.into())),
        "identity_reference_v1"=>reference_from_detjson(payload).map(|value|ResourceValue::IdentityReference(value.into())),
        "bytes_v1" => bytes_value::parse_bytes_v1(obj),
        "tuple_v1" | "rational_v1" => tuple_value::candidate_with_budget(payload,budget,contracts,references,catalog),
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
        "list" | "set" | "map" | "record" => unreachable!("structural restore is owned by heap jobs"),
        _ => Err(format!("지원하지 않는 value_det type: {kind}")),
    }
}

fn parse_record_field_types(payload: &JsonValue) -> Result<BTreeMap<String, RecordFieldType>, String> {
    // F5-GA[WIRE_TYPE_JOBS] The initial Record job and its immutable children reduce to exactly one root Record, including the empty fields case; failed children return before the final pop.
    let bad = || "E_PRODUCT_RECORD_WIRE: 올바르지 않은 묶음 저장값입니다".to_string();
    enum Job<'a> { Type(&'a JsonValue), Applied(&'a str, usize), Record(&'a serde_json::Map<String, JsonValue>) }
    let fields = payload.as_object().ok_or_else(bad)?;
    let mut jobs = vec![Job::Record(fields)];
    jobs.extend(fields.values().rev().map(Job::Type));
    let mut built: Vec<RecordFieldType> = Vec::new();
    while let Some(job) = jobs.pop() {
        match job {
            Job::Type(value) => {
                let obj = value.as_object().ok_or_else(bad)?;
                let kind = obj.get("kind").and_then(JsonValue::as_str).ok_or_else(bad)?;
                let keys: &[&str] = match kind {
                    "Named" => &["kind", "name"],
                    "Applied" => &["kind", "name", "args"],
                    "Record" => &["kind", "fields"],
                    _ => return Err(bad()),
                };
                if obj.len() != keys.len() || keys.iter().any(|key| !obj.contains_key(*key)) { return Err(bad()); }
                match kind {
                    "Named" => built.push(RecordFieldType::Named { name: obj["name"].as_str().ok_or_else(bad)?.into() }),
                    "Applied" => {
                        let name = obj["name"].as_str().ok_or_else(bad)?;
                        let args = obj["args"].as_array().ok_or_else(bad)?;
                        jobs.push(Job::Applied(name, args.len()));
                        jobs.extend(args.iter().rev().map(Job::Type));
                    }
                    "Record" => {
                        let fields = obj["fields"].as_object().ok_or_else(bad)?;
                        jobs.push(Job::Record(fields));
                        jobs.extend(fields.values().rev().map(Job::Type));
                    }
                    _ => unreachable!(),
                }
            }
            Job::Applied(name, count) => {
                let args = built.split_off(built.len() - count);
                built.push(RecordFieldType::Applied { name: name.into(), args });
            }
            Job::Record(fields) => {
                let values = built.split_off(built.len() - fields.len());
                built.push(RecordFieldType::Record { fields: fields.keys().cloned().zip(values).collect() });
            }
        }
    }
    let mut result = built.pop().expect("projected record field types");
    let RecordFieldType::Record { fields } = &mut result else { unreachable!() };
    Ok(std::mem::take(fields))
}

fn parse_record_header(obj: &serde_json::Map<String, JsonValue>) -> Result<(BTreeMap<String, RecordFieldType>, &[JsonValue]), String> {
    let bad = || "E_PRODUCT_RECORD_WIRE: 올바르지 않은 묶음 저장값입니다".to_string();
    if obj.len() != 4 || obj.keys().any(|key| !["type", "version", "type_identity", "fields"].contains(&key.as_str())) {
        return Err(bad());
    }
    if obj.get("version").and_then(JsonValue::as_u64) != Some(1) {
        return Err("E_PRODUCT_RECORD_VERSION: 지원하지 않는 묶음 저장 판번호입니다".into());
    }
    let field_types = parse_record_field_types(obj.get("type_identity").ok_or_else(bad)?)?;
    let rows = obj.get("fields").and_then(JsonValue::as_array).ok_or_else(bad)?;
    Ok((field_types, rows))
}

// Existing Map compatibility decoding is unchanged. A new record cannot use
// the old scalar/collection defaults to repair a malformed field payload.
fn validate_record_wire_value(value: &JsonValue) -> Result<(), String> {
    let bad = || "E_PRODUCT_RECORD_WIRE: 묶음 필드 저장값이 올바르지 않습니다".to_string();
    enum Job<'a> { Value(&'a JsonValue), Entry(&'a JsonValue) }
    let mut jobs = vec![Job::Value(value)];
    while let Some(job) = jobs.pop() {
    let value = match job {
        Job::Value(value) => value,
        Job::Entry(entry) => {
            let entry = entry.as_object().ok_or_else(bad)?;
            if entry.len() != 2 { return Err(bad()); }
            jobs.push(Job::Value(entry.get("value").ok_or_else(bad)?));
            jobs.push(Job::Value(entry.get("key").ok_or_else(bad)?));
            continue;
        }
    };
    let obj = value.as_object().ok_or_else(bad)?;
    let kind = obj.get("type").and_then(JsonValue::as_str).ok_or_else(bad)?;
    let keys: &[&str] = match kind {
        "identity_reference_v1"=> &["type","profile_version","target_type_identity_hex","owner_space_identity_hex","target_identity_hex","generation_u64"],
        "registered_atom_v1" => &["type", "profile_version", "owner_registry_identity_hex", "registered_atom_identity_hex", "contract_profile", "contract_version", "contract_snapshot_identity_hex", "contract_sha256"],
        "bytes_v1" => &bytes_value::BYTES_FIELDS,
        "tuple_v1" | "rational_v1" => &tuple_value::FIELDS,
        "none" => &["type"],
        "bool" | "string" | "handle" => &["type", "value"],
        "fixed64" => &["type", "raw_i64"],
        "list" | "set" => &["type", "items"],
        "map" => &["type", "entries"],
        "unit" => &["type", "schema", "raw_i64", "dimension"],
        "record" => &["type", "version", "type_identity", "fields"],
        _ => return Err(bad()),
    };
    if obj.len() != keys.len() || keys.iter().any(|key| !obj.contains_key(*key)) { return Err(bad()); }
    match kind {
        "bool" if !obj["value"].is_boolean() => return Err(bad()),
        "string" | "handle" if !obj["value"].is_string() => return Err(bad()),
        "fixed64" => {
            let raw = obj["raw_i64"].as_str().ok_or_else(bad)?;
            if raw.parse::<i64>().map(|number| number.to_string()).ok().as_deref() != Some(raw) { return Err(bad()); }
        }
        "list" | "set" => jobs.extend(obj["items"].as_array().ok_or_else(bad)?.iter().rev().map(Job::Value)),
        "map" => jobs.extend(obj["entries"].as_array().ok_or_else(bad)?.iter().rev().map(Job::Entry)),
        _ => {}
    }
    }
    Ok(())
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
    // F5-GA[WIRE_UNIT_AXIS] dimensions.len()==AXIS_COUNT was checked immediately above; enumerate therefore stays inside this fixed array.
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
    fn snapshot_json_owner_compares_and_drops_deep_roots_without_recursion() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let deep = |leaf| {
                let mut value = JsonValue::Bool(leaf);
                for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
                SnapshotJsonOwner(JsonValue::Object([("tree".into(), value)].into_iter().collect()))
            };
            let original = deep(true);
            let copied = original.clone();
            assert!(original == copied);
            assert_eq!(copied.compact_text(), original.compact_text());
            drop(copied);
            let equal = deep(true);
            let different = deep(false);
            let same = original == equal;
            let changed = original != different;
            drop(original); drop(equal); drop(different);
            let mut locals = BTreeMap::from([("saved", deep(true))]);
            locals.insert("saved", SnapshotJsonOwner(JsonValue::Null));
            let unwind = std::panic::catch_unwind(|| { let _staged = deep(true); panic!("JSON owner cleanup test"); });
            assert!(same && changed && unwind.is_err());
        }).unwrap().join().unwrap();
    }

    #[test]
    fn snapshot_owner_clone_and_compact_writer_preserve_existing_json_bytes() {
        for value in [json!({"a":[null,true,1.25,"한글\\\n"],"z":{}}),
            resource_value_to_detjson(&ResourceValue::Bytes(ddonirang_core::bytes::ByteSequence::new(vec![0,255])))] {
            let expected = value.to_string();
            let owner = SnapshotJsonOwner::from_value(value);
            assert_eq!(owner.clone().compact_text(), expected);
            let snapshot = serialize_resource_snapshot_json(&owner);
            assert_eq!(snapshot.trim_end_matches('\n'), expected);
            if owner.get("type").and_then(JsonValue::as_str) == Some("bytes_v1") { assert!(snapshot.ends_with('\n')); }
        }
    }

    #[test]
    fn snapshot_json_parser_keeps_late_failure_and_terminal_error_behavior() {
        let nested = "[".repeat(80) + "true" + &"]".repeat(80);
        let valid = format!("{{\"a\":{nested}}}");
        let parsed = parse_snapshot_json(&valid).unwrap();
        assert!(parsed.equals_json(&serde_json::from_str::<JsonValue>(&valid).unwrap()));
        for raw in [format!("{{\"a\":{nested},\"z\":[}}"),
            format!("{{\"a\":{nested},\"a\":0}}"), format!("{valid} trailing"),
            format!("{{\"a\":{nested},\"z\":{{\"type\":\"bytes_v1\"}}}}") ] {
            assert!(parse_snapshot_json(&raw).is_err());
        }
    }

    #[test]
    fn saved_json_projection_keeps_record_types_and_mixed_collection_shape() {
        let types = BTreeMap::from([
            ("a".into(), RecordFieldType::Applied { name: "차림".into(), args: vec![RecordFieldType::Named { name: "글".into() }] }),
            ("b".into(), RecordFieldType::Record { fields: [("c".into(), RecordFieldType::Named { name: "참거짓".into() })].into() }),
        ]);
        let expected_types = serde_json::to_value(&types).unwrap();
        let value = ResourceValue::Record { field_types: types,
            fields: [("a".into(), ResourceValue::List(vec![ResourceValue::String("한글".into())])),
                ("b".into(), ResourceValue::Map([("index".into(), ResourceMapEntry { key: ResourceValue::String("c".into()), value: ResourceValue::Bool(true) })].into()))].into(),
        };
        let expected = json!({"type":"record", "version":1, "type_identity":expected_types,
            "fields":[{"name":"a", "value":{"type":"list", "items":[{"type":"string", "value":"한글"}]}},
                {"name":"b", "value":{"type":"map", "entries":[{"key":{"type":"string", "value":"c"}, "value":{"type":"bool", "value":true}}]}}]});
        assert_eq!(resource_value_to_detjson(&value), expected);
        let set = ResourceValue::Set([("z".into(), ResourceValue::None), ("a".into(), ResourceValue::Bool(true))].into());
        assert_eq!(resource_value_to_detjson(&set), json!({"type":"set","items":[{"type":"bool","value":true},{"type":"none"}]}));
    }

    #[test]
    fn saved_record_type_parser_preserves_serde_admission_without_json_clone() {
        for value in [json!({}), json!({"a":{"kind":"Named","name":""}}),
            json!({"a":{"kind":"Applied","name":"차림","args":[]}}),
            json!({"a":{"kind":"Record","fields":{"x":{"kind":"Named","name":"글"}}}}),
            json!({"a":{"kind":"Named","name":"글","extra":0}}),
            json!({"a":{"kind":"Named"}}), json!({"a":{"kind":"Named","name":1}}),
            json!({"a":{"kind":"Applied","name":"차림","args":null}}),
            json!({"a":{"kind":"Applied","name":"차림","args":[{"kind":"Unknown"}]}}),
            json!({"a":{"kind":"Record","fields":[]}}), json!([]), json!(null)] {
            let original = serde_json::from_value::<BTreeMap<String, RecordFieldType>>(value.clone());
            let parsed = parse_record_field_types(&value);
            assert_eq!(parsed.is_ok(), original.is_ok(), "{value}");
            if let (Ok(parsed), Ok(original)) = (parsed, original) { assert_eq!(parsed, original); }
        }
    }

    #[test]
    fn saved_json_projection_handles_deep_record_values_and_types_on_small_stack() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::String("leaf".into());
            let mut kind = RecordFieldType::Named { name: "글".into() };
            for _ in 0..20_000 {
                value = ResourceValue::List(vec![value]);
                kind = RecordFieldType::Applied { name: "차림".into(), args: vec![kind] };
            }
            let record = ResourceValue::Record { field_types: [("a".into(), kind)].into(), fields: [("a".into(), value)].into() };
            let projected = resource_value_to_detjson(&record);
            // This exercises the admitted-value worker, not the still-bounded
            // raw/custody frontdoor. Do not infer new depth admission from it.
            let restored = resource_value_from_admitted_detjson(&projected).unwrap();
            let values_identical = restored == record;
            drop(restored);
            let parsed_types = parse_record_field_types(&projected["type_identity"]).unwrap();
            let ResourceValue::Record { field_types, .. } = &record else { unreachable!() };
            let types_identical = &parsed_types == field_types;
            let fields_valid = validate_record_wire_value(&projected["fields"][0]["value"]).is_ok();
            drop(parsed_types);
            let mut late_invalid = serde_json::Map::new();
            // Move rather than clone the deep fixture's projected type.
            let mut projected = projected;
            let identity = projected["type_identity"].as_object_mut().unwrap().remove("a").unwrap();
            late_invalid.insert("a".into(), identity);
            late_invalid.insert("z".into(), json!({"kind":"Named","name":true}));
            let mut late_invalid = JsonValue::Object(late_invalid);
            let rejected = parse_record_field_types(&late_invalid).is_err();
            let identity = late_invalid.as_object_mut().unwrap().remove("a").unwrap();
            projected["type_identity"].as_object_mut().unwrap().insert("a".into(), identity);
            let text = serialize_resource_snapshot_json(&projected);
            let mut expected = "{\"fields\":[{\"name\":\"a\",\"value\":".to_string();
            expected.push_str(&"{\"items\":[".repeat(20_000));
            expected.push_str("{\"type\":\"string\",\"value\":\"leaf\"}");
            expected.push_str(&"],\"type\":\"list\"}".repeat(20_000));
            expected.push_str("}],\"type\":\"record\",\"type_identity\":{\"a\":");
            expected.push_str(&"{\"args\":[".repeat(20_000));
            expected.push_str("{\"kind\":\"Named\",\"name\":\"글\"}");
            expected.push_str(&"],\"kind\":\"Applied\",\"name\":\"차림\"}".repeat(20_000));
            expected.push_str("},\"version\":1}");
            // Adopt the projected fixture into the product root owner. Other
            // projection callers still need their own lifetime migration.
            drop(SnapshotJsonOwner(projected));
            drop(record);
            assert!(values_identical && types_identical && fields_valid && rejected);
            assert_eq!(text, expected);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn saved_json_restore_keeps_map_error_order_and_cleans_deep_partial_values() {
        assert_eq!(resource_value_from_admitted_detjson(&json!({"type":"map","entries":[
            {"key":{"type":"bool","value":1}}, null
        ]})).unwrap_err(), "bool value 형식 오류: 불리언이 필요합니다");
        assert_eq!(resource_value_from_admitted_detjson(&json!({"type":"map","entries":[
            {"key":{"type":"none"}}, null
        ]})).unwrap_err(), "map entry value 누락");
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            for map in [false, true] {
                let mut deep = ResourceValue::None;
                for _ in 0..20_000 { deep = ResourceValue::List(vec![deep]); }
                let deep = resource_value_to_detjson(&deep);
                let payload = if map {
                    JsonValue::Object([("type".into(), "map".into()), ("entries".into(), JsonValue::Array(vec![
                        JsonValue::Object([("key".into(), deep)].into_iter().collect()),
                    ]))].into_iter().collect())
                } else {
                    JsonValue::Object([("type".into(), "list".into()), ("items".into(), JsonValue::Array(vec![
                        deep, json!({"type":"bool","value":1}),
                    ]))].into_iter().collect())
                };
                let error = resource_value_from_admitted_detjson(&payload).unwrap_err();
                let mut pending = vec![payload];
                while let Some(mut value) = pending.pop() {
                    match &mut value {
                        JsonValue::Array(items) => pending.extend(std::mem::take(items)),
                        JsonValue::Object(items) => pending.extend(std::mem::take(items).into_iter().map(|(_, value)| value)),
                        _ => {},
                    }
                }
                assert_eq!(error, if map { "map entry value 누락" } else { "bool value 형식 오류: 불리언이 필요합니다" });
            }
        }).unwrap().join().unwrap();
    }

    #[test]
    fn identity_tree_iterative_walk_has_no_depth_gate_and_keeps_first_error() {
        let nested = |depth: usize| {
            let mut value = JsonValue::Null;
            for _ in 0..depth { value = JsonValue::Array(vec![value]); }
            SnapshotJsonOwner::from_value(value)
        };
        assert!(verify_identity_tree(&nested(128), &[], None, 0).is_ok());
        assert!(verify_identity_tree(&nested(20_000), &[], None, 0).is_ok());
        let wrong_atom = json!({"type": "registered_atom_v1"});
        let atom_error = verify_identity_tree(&wrong_atom, &[], None, 0).unwrap_err();
        let atom_first = SnapshotJsonOwner::from_value(JsonValue::Object([
            ("a".into(), wrong_atom), ("z".into(), nested(20_000).into_parent_value()),
        ].into_iter().collect()));
        assert_eq!(verify_identity_tree(&atom_first, &[], None, 0).unwrap_err(), atom_error);
        let depth_first = SnapshotJsonOwner::from_value(JsonValue::Object([
            ("a".into(), nested(20_000).into_parent_value()), ("z".into(), json!({"type":"registered_atom_v1"})),
        ].into_iter().collect()));
        assert_eq!(verify_identity_tree(&depth_first, &[], None, 0).unwrap_err(), atom_error);
        let array = SnapshotJsonOwner::from_value(JsonValue::Array(vec![
            json!({"type":"registered_atom_v1"}), nested(20_000).into_parent_value(),
        ]));
        assert_eq!(verify_identity_tree(&array, &[], None, 0).unwrap_err(), atom_error);
    }

    #[test]
    fn beta1_reference_json_checks_closed_tuple_and_custody_before_collection_keys(){
        use crate::subject_reference_owner::{SavedSubjectRow,SavedWorldIdentityCustodyV1,SubjectReferenceContextV1};
        let reference=ddonirang_core::platform::IdentityReferenceValueV1{profile_version:1,target_type_identity:vec![0x11,0],owner_space_identity:vec![2;32],target_identity:vec![3;32],generation:0};
        let custody=SavedWorldIdentityCustodyV1{nonce:[1;32],world:[2;32],pesi:[4;32],next_subject_ordinal:1,
            rows:vec![SavedSubjectRow{identity:[3;32],ordinal:0,live:true,target_type:vec![0x11,0]}]};
        let context=SubjectReferenceContextV1::new(&custody).unwrap();
        let wire=reference_to_detjson(&reference);assert_eq!(wire.as_object().unwrap().len(),6);
        assert_eq!(reference_from_detjson(&wire).unwrap(),reference);
        assert!(resource_value_from_detjson(&wire).is_err());
        assert_eq!(resource_value_from_detjson_with_custody(&wire,&[],Some(&context)).unwrap(),ResourceValue::IdentityReference(reference.clone().into()));
        for key in wire.as_object().unwrap().keys() {
            let mut missing=wire.clone();missing.as_object_mut().unwrap().remove(key);assert!(reference_from_detjson(&missing).is_err());
        }
        for (key,value) in [("generation_u64",json!("00")),("generation_u64",json!(0)),("profile_version",json!(2)),
            ("target_type_identity_hex",json!("AB")),("target_type_identity_hex",json!("11".repeat(4097))),
            ("owner_space_identity_hex",json!("02")),("target_identity_hex",json!("03".repeat(33)))] {
            let mut wrong=wire.clone();wrong[key]=value;assert!(reference_from_detjson(&wrong).is_err(),"{key}");
        }
        let mut wrong=wire.clone();wrong["generation_u64"]=json!("1");
        for kind in ["list","set"] {
            for items in [json!([wire,wrong]),json!([wrong,wire])] {
                let nested=json!({"type":kind,"items":items});
                assert!(resource_value_from_detjson_with_custody(&nested,&[],Some(&context)).is_err());
            }
        }
        let extra=json!({"type":"identity_reference_v1","profile_version":1,"target_type_identity_hex":"1100", "owner_space_identity_hex":"02".repeat(32),"target_identity_hex":"03".repeat(32),"generation_u64":"0","repair":true});
        assert!(reference_from_detjson(&extra).is_err());
    }

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
