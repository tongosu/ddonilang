//! RD-674 saved-value owner with RD-682 bounded typed element admission.
//!
//! Structural candidates are distinct from admitted immutable Tensor values.
//! TYPE-I64 fixture bytes never become a runtime type. No world publication,
//! public transport spelling, or adapter-local encoder is defined here.

use sha2::{Digest, Sha256};
use ddonirang_core::{Fixed64, platform::ResourceValue};
use ddonirang_lang::ast::TypeRef;
use crate::basic_tensor_element_type_identity as element_identity;

const TAG: u8 = 0x0e;
const PROFILE: &[u8] = b"DDN-BASIC-TENSOR-VALUE-V1\0";
const FAMILY: &[u8] = b"ddn.tensor.basic";

// RD-683: private transport remains inside the sole unit_wire owner. Raw
// map visitation preserves duplicate/order evidence before any Value collapse.
const TRANSPORT_KEYS: [&str; 8] = ["carrier_base64url", "carrier_byte_length",
    "carrier_sha256", "resource_value_byte_length", "resource_value_sha256",
    "resource_value_tag", "type", "version"];
const CARRIER_MAX: usize = 1_000_000;
const BASE64_MAX: usize = 1_333_334;
// A3-P6 provisional product profile ceiling, not a Tensor semantic rank limit
// (RD-386). Bound recursive collection materialization before allocation.
const PROFILE_MAX_RANK: usize = 64;
const PROFILE_COLLECTION_MAX_NODES: u64 = 1_000_000;

pub(super) fn preflight_collection(shape: &[u64]) -> Result<(), TensorFrameError> {
    // A3-P6: a tiny zero-element carrier can otherwise request billions of
    // empty nested lists. Check realized containers/leaves, not element count.
    if shape.len() > PROFILE_MAX_RANK { return Err(TensorFrameError::ResourceLimit); }
    let mut prefix = 1u64;
    let mut nodes = 1u64;
    for axis in shape {
        prefix = prefix.checked_mul(*axis).ok_or(TensorFrameError::ResourceLimit)?;
        nodes = nodes.checked_add(prefix).ok_or(TensorFrameError::ResourceLimit)?;
        if nodes > PROFILE_COLLECTION_MAX_NODES { return Err(TensorFrameError::ResourceLimit); }
    }
    Ok(())
}

pub(super) fn validate_transport_keys<'a>(keys: impl Iterator<Item = &'a str>) -> Result<(), TensorFrameError> {
    if keys.eq(TRANSPORT_KEYS) { Ok(()) } else { Err(TensorFrameError::Fields) }
}

#[test]
fn vc14_rd683_raw_transport_rejects_ambiguous_or_corrupt_leaf() {
    let kind = TypeRef::Named("셈수".into());
    for (shape, elements) in [(vec![], vec![Fixed64::from_raw_i64(7)]),
        (vec![0], vec![]), (vec![2, 2], vec![Fixed64::from_raw_i64(-1); 4])] {
        let tensor = encode_typed_tensor(&kind, &shape, &elements).unwrap();
        let value = transport_value(&tensor);
        let raw = serde_json::to_string(&value).unwrap();
        assert_eq!(transport_from_raw(&raw).unwrap(), tensor);
        assert_eq!(transport_value(&transport_from_raw(&raw).unwrap()), value);
        for (key, replacement) in [
            ("version", serde_json::json!(1.0)), ("version", serde_json::json!("1")),
            ("resource_value_tag", serde_json::json!("0E")), ("type", serde_json::json!("list")),
            ("carrier_byte_length", serde_json::json!(1_000_001)),
            ("resource_value_byte_length", serde_json::json!(0)),
            ("carrier_sha256", serde_json::json!("0".repeat(64))),
            ("resource_value_sha256", serde_json::json!("0".repeat(64))),
            ("carrier_base64url", serde_json::json!(format!("{}=", value["carrier_base64url"].as_str().unwrap()))),
        ] {
            let mut wrong = value.clone(); wrong[key] = replacement;
            assert!(transport_from_raw(&wrong.to_string()).is_err(), "accepted {key}");
        }
        let duplicate = raw.replacen("{", "{\"version\":1,", 1);
        assert!(transport_from_raw(&duplicate).is_err());
        let reordered = format!("{{\"version\":1,{}", raw[1..].replace(",\"version\":1}", "}"));
        assert!(transport_from_raw(&reordered).is_err());
        assert!(transport_from_raw(&format!("{raw}{{}}")).is_err());
        let mut missing = value.clone(); missing.as_object_mut().unwrap().remove("carrier_sha256");
        assert!(transport_from_raw(&missing.to_string()).is_err());
        let mut unknown = value.clone(); unknown["unknown"] = serde_json::json!(0);
        assert!(transport_from_raw(&unknown.to_string()).is_err());
        let mut trailing = tensor.saved_bytes().to_vec(); trailing.push(0);
        let mut wrong = value.clone();
        wrong["carrier_base64url"] = serde_json::json!(base64url(&trailing[1..]));
        wrong["carrier_byte_length"] = serde_json::json!(trailing.len() - 1);
        wrong["resource_value_byte_length"] = serde_json::json!(trailing.len());
        wrong["carrier_sha256"] = serde_json::json!(sha_hex(&trailing[1..]));
        wrong["resource_value_sha256"] = serde_json::json!(sha_hex(&trailing));
        assert!(transport_from_raw(&wrong.to_string()).is_err());
    }
}

fn base64url(bytes: &[u8]) -> String {
    ddonirang_core::commit_canon::carrier_base64url(bytes)
}

pub(super) fn from_carrier(carrier: &[u8]) -> Result<AdmittedTensor, TensorFrameError> {
    if carrier.len() > CARRIER_MAX { return Err(TensorFrameError::Length); }
    let mut saved = vec![TAG]; saved.extend_from_slice(carrier);
    admit_tensor_value(&saved, saved.len(), &Sha256::digest(carrier).into(), &Sha256::digest(&saved).into())
}

pub(super) fn decode_base64url(text: &str) -> Result<Vec<u8>, TensorFrameError> {
    if text.len() > BASE64_MAX || text.len() % 4 == 1 { return Err(TensorFrameError::Length); }
    let mut out = Vec::new();
    out.try_reserve_exact(text.len() * 3 / 4).map_err(|_| TensorFrameError::Capacity)?;
    let mut bits = 0u32;
    let mut count = 0;
    for byte in text.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A', b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52, b'-' => 62, b'_' => 63,
            _ => return Err(TensorFrameError::Integrity),
        };
        bits = (bits << 6) | u32::from(value);
        count += 6;
        if count >= 8 { count -= 8; out.push((bits >> count) as u8); }
    }
    if base64url(&out) != text { return Err(TensorFrameError::Integrity); }
    Ok(out)
}

fn sha_hex(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

pub(super) fn transport_value(value: &AdmittedTensor) -> serde_json::Value {
    // F5-GA[TENSOR_ADMITTED]: Private admitted owners retain a structurally verified leading tag; shared saved_bytes cannot be shortened by this projection.
    let carrier = &value.saved_bytes()[1..];
    serde_json::json!({
        "carrier_base64url": base64url(carrier), "carrier_byte_length": carrier.len(),
        "carrier_sha256": sha_hex(carrier), "resource_value_byte_length": carrier.len() + 1,
        "resource_value_sha256": sha_hex(value.saved_bytes()), "resource_value_tag": "0e",
        "type": "tensor_v1", "version": 1,
    })
}

pub(super) fn transport_from_value(value: &serde_json::Value) -> Result<AdmittedTensor, TensorFrameError> {
    let obj = value.as_object().ok_or(TensorFrameError::Fields)?;
    if !obj.keys().map(String::as_str).eq(TRANSPORT_KEYS) { return Err(TensorFrameError::Fields); }
    if obj["type"].as_str() != Some("tensor_v1") || obj["resource_value_tag"].as_str() != Some("0e") {
        return Err(TensorFrameError::Tag);
    }
    if obj["version"].as_u64() != Some(1) { return Err(TensorFrameError::Version); }
    let length = obj["carrier_byte_length"].as_u64().ok_or(TensorFrameError::Length)?;
    if length > CARRIER_MAX as u64 || obj["resource_value_byte_length"].as_u64() != length.checked_add(1) {
        return Err(TensorFrameError::Length);
    }
    let encoded = obj["carrier_base64url"].as_str().ok_or(TensorFrameError::Fields)?;
    let carrier = decode_base64url(encoded)?;
    if carrier.len() as u64 != length || obj["carrier_sha256"].as_str() != Some(sha_hex(&carrier).as_str()) {
        return Err(TensorFrameError::Integrity);
    }
    let mut saved = Vec::with_capacity(carrier.len() + 1);
    saved.push(TAG);
    saved.extend_from_slice(&carrier);
    let candidate = inspect_tensor_frame(&saved)?;
    validate_tensor_elements(&candidate)?;
    if obj["resource_value_sha256"].as_str() != Some(sha_hex(&saved).as_str()) {
        return Err(TensorFrameError::Integrity);
    }
    admit_tensor_value(&saved, saved.len(), &Sha256::digest(&carrier).into(), &Sha256::digest(&saved).into())
}

/// Standalone leaf entry for raw snapshot consumers; callers additionally
/// enforce their existing whole-snapshot ceiling before parsing the container.
pub(super) fn transport_from_raw(raw: &str) -> Result<AdmittedTensor, TensorFrameError> {
    use serde::de::{MapAccess, Visitor, Error};
    struct OrderedLeaf;
    impl<'de> Visitor<'de> for OrderedLeaf {
        type Value = serde_json::Value;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("정본 Tensor 저장 객체") }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut obj = serde_json::Map::new();
            for expected in TRANSPORT_KEYS {
                let key = map.next_key::<String>()?.ok_or_else(|| M::Error::custom("Tensor 키 누락"))?;
                if key != expected { return Err(M::Error::custom("Tensor 키 중복 또는 순서 오류")); }
                obj.insert(key, map.next_value::<serde_json::Value>()?);
            }
            if map.next_key::<String>()?.is_some() { return Err(M::Error::custom("Tensor 추가 키")); }
            Ok(serde_json::Value::Object(obj))
        }
    }
    // 8 bounded scalars; reject excessive raw input before allocating strings.
    if raw.len() > BASE64_MAX + 1024 { return Err(TensorFrameError::Length); }
    let mut decoder = serde_json::Deserializer::from_str(raw);
    let value = serde::Deserializer::deserialize_map(&mut decoder, OrderedLeaf).map_err(|_| TensorFrameError::Fields)?;
    decoder.end().map_err(|_| TensorFrameError::Fields)?;
    transport_from_value(&value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TensorFrameError {
    ResourceLimit,
    Tag,
    Profile,
    Version,
    Fields,
    Length,
    Integrity,
    EmptyType,
    Rank,
    ShapeOverflow,
    Count,
    EmptyElement,
    Capacity,
    ElementType(element_identity::ElementTypeError),
    ElementValue,
    ShapeMismatch,
}

/// Immutable borrowed candidate, deliberately not a semantic Tensor value.
/// Construction is private: every byte range and count is checked first.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct UnadmittedTensorFrame<'a> {
    saved: &'a [u8],
    element_type: &'a [u8],
    rank: u32,
    axes: &'a [u8],
    element_count: u64,
    payloads: &'a [u8],
}

impl<'a> UnadmittedTensorFrame<'a> {
    // F5-GA[TENSOR_CANDIDATE]: The sole structural constructor validates tag, exact axes, all VALUE32 boundaries and EOF before exposing these immutable borrowed ranges.
    pub(super) fn saved_bytes(&self) -> &'a [u8] {
        self.saved
    }
    pub(super) fn carrier_bytes(&self) -> &'a [u8] {
        &self.saved[1..]
    }
    pub(super) fn element_type_bytes(&self) -> &'a [u8] {
        self.element_type
    }
    pub(super) fn rank(&self) -> u32 {
        self.rank
    }
    pub(super) fn element_count(&self) -> u64 {
        self.element_count
    }
    pub(super) fn shape(&self) -> impl Iterator<Item = u64> + '_ {
        self.axes
            .chunks_exact(8)
            .map(|axis| u64::from_be_bytes(axis.try_into().unwrap()))
    }
    pub(super) fn elements(&self) -> impl Iterator<Item = &'a [u8]> {
        // The structural constructor already proved every VALUE32 boundary.
        let mut cursor = Cursor(self.payloads);
        (0..self.element_count).map(move |_| {
            let length = cursor.u32().expect("validated VALUE32") as usize;
            cursor.take(length).expect("validated VALUE32")
        })
    }
}

struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], TensorFrameError> {
        if length > self.0.len() {
            return Err(TensorFrameError::Length);
        }
        let (head, tail) = self.0.split_at(length);
        self.0 = tail;
        Ok(head)
    }
    fn u32(&mut self) -> Result<u32, TensorFrameError> {
        // F5-GA[TENSOR_U32]: take(4) returns exactly four bytes or Err before array conversion.
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn field(&mut self, tag: u8) -> Result<&'a [u8], TensorFrameError> {
        if self.take(1)? != [tag] {
            return Err(TensorFrameError::Fields);
        }
        let length = usize::try_from(self.u32()?).map_err(|_| TensorFrameError::Length)?;
        self.take(length)
    }
    fn finish(self) -> Result<(), TensorFrameError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(TensorFrameError::Length)
        }
    }
}

fn shape_count(mut axes: impl Iterator<Item = u64> + Clone) -> Result<u64, TensorFrameError> {
    // A later zero axis must not depend on the order of overflow checks.
    if axes.clone().any(|axis| axis == 0) {
        return Ok(0);
    }
    axes.try_fold(1u64, |count, axis| {
        count
            .checked_mul(axis)
            .ok_or(TensorFrameError::ShapeOverflow)
    })
}

/// Checks framing and arithmetic without allocating from untrusted counts.
/// Callers must not publish this candidate as an admitted runtime value.
pub(super) fn inspect_tensor_frame(
    saved: &[u8],
) -> Result<UnadmittedTensorFrame<'_>, TensorFrameError> {
    // F5-GA[TENSOR_FIELDS]: Rank/count exact-width checks and chunks_exact(8) precede the corresponding fixed-array conversions.
    let mut cursor = Cursor(saved);
    if cursor.take(1)? != [TAG] {
        return Err(TensorFrameError::Tag);
    }
    if cursor.take(PROFILE.len())? != PROFILE {
        return Err(TensorFrameError::Profile);
    }
    if cursor.take(2)? != [0, 1] {
        return Err(TensorFrameError::Version);
    }
    if cursor.take(1)? != [7] {
        return Err(TensorFrameError::Fields);
    }
    if cursor.field(1)? != FAMILY {
        return Err(TensorFrameError::Profile);
    }
    if cursor.field(2)? != [0, 1] {
        return Err(TensorFrameError::Version);
    }
    let element_type = cursor.field(3)?;
    if element_type.is_empty() {
        return Err(TensorFrameError::EmptyType);
    }
    let rank = cursor.field(4)?;
    if rank.len() != 4 {
        return Err(TensorFrameError::Rank);
    }
    let rank = u32::from_be_bytes(rank.try_into().unwrap());
    let mut shape = Cursor(cursor.field(5)?);
    if shape.u32()? != rank {
        return Err(TensorFrameError::Rank);
    }
    let shape_bytes = usize::try_from(rank)
        .ok()
        .and_then(|rank| rank.checked_mul(8))
        .ok_or(TensorFrameError::Length)?;
    let axes = shape.take(shape_bytes)?;
    shape.finish()?;
    let count = cursor.field(6)?;
    if count.len() != 8 {
        return Err(TensorFrameError::Count);
    }
    let element_count = u64::from_be_bytes(count.try_into().unwrap());
    let actual_count = shape_count(
        axes.chunks_exact(8)
            .map(|axis| u64::from_be_bytes(axis.try_into().unwrap())),
    )?;
    if actual_count != element_count {
        return Err(TensorFrameError::Count);
    }
    let mut elements = Cursor(cursor.field(7)?);
    if u64::from(elements.u32()?) != element_count {
        return Err(TensorFrameError::Count);
    }
    let payloads = elements.0;
    // Each VALUE32 requires four length bytes plus nonempty canonical bytes.
    if element_count > (payloads.len() / 5) as u64 {
        return Err(TensorFrameError::Length);
    }
    for _ in 0..element_count {
        let length = usize::try_from(elements.u32()?).map_err(|_| TensorFrameError::Length)?;
        if length == 0 {
            return Err(TensorFrameError::EmptyElement);
        }
        elements.take(length)?;
    }
    elements.finish()?;
    cursor.finish()?;
    Ok(UnadmittedTensorFrame {
        saved,
        element_type,
        rank,
        axes,
        element_count,
        payloads,
    })
}

/// Sealed length and the two distinct digest domains are checked as evidence,
/// not used as replacements for exact type/value byte admission or equality.
pub(super) fn inspect_sealed_tensor_frame<'a>(
    saved: &'a [u8],
    expected_length: usize,
    carrier_hash: &[u8; 32],
    saved_hash: &[u8; 32],
) -> Result<UnadmittedTensorFrame<'a>, TensorFrameError> {
    if saved.len() != expected_length {
        return Err(TensorFrameError::Length);
    }
    let frame = inspect_tensor_frame(saved)?;
    let actual_carrier: [u8; 32] = Sha256::digest(frame.carrier_bytes()).into();
    let actual_saved: [u8; 32] = Sha256::digest(saved).into();
    if &actual_carrier != carrier_hash || &actual_saved != saved_hash {
        return Err(TensorFrameError::Integrity);
    }
    Ok(frame)
}

/// An immutable admitted Tensor, distinct from scalar and nested collection.
/// RD-682 currently admits only the existing canonical Fixed64/셈수 owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AdmittedTensor {
    shape: Vec<u64>,
    elements: Vec<Fixed64>,
    saved: Vec<u8>,
}

impl AdmittedTensor {
    pub(super) fn into_runtime(self) -> Result<ddonirang_lang::runtime::TensorValue, TensorFrameError> {
        let candidate = inspect_tensor_frame(&self.saved)?;
        validate_tensor_elements(&candidate)?;
        let identity = candidate.element_type_bytes().to_vec();
        Ok(ddonirang_lang::runtime::TensorValue::from_admitted_parts(identity, self.shape, self.elements, self.saved))
    }
    pub(super) fn shape(&self) -> &[u64] { &self.shape }
    pub(super) fn elements(&self) -> &[Fixed64] { &self.elements }
    pub(super) fn saved_bytes(&self) -> &[u8] { &self.saved }

    /// Prepare a replacement without mutating the previously admitted value.
    /// Complete admission, including exact EOF, precedes shape comparison.
    pub(super) fn prepare_replacement(
        &self, saved: &[u8], carrier_hash: &[u8; 32], saved_hash: &[u8; 32],
    ) -> Result<Self, TensorFrameError> {
        let candidate = inspect_sealed_tensor_frame(saved, saved.len(), carrier_hash, saved_hash)?;
        validate_tensor_elements(&candidate)?;
        if !self.shape.iter().copied().eq(candidate.shape()) {
            return Err(TensorFrameError::ShapeMismatch);
        }
        admit_tensor_value(saved, saved.len(), carrier_hash, saved_hash)
    }
}

pub(super) fn verify_runtime(value: &ddonirang_lang::runtime::TensorValue) -> Result<(), TensorFrameError> {
    if value.shape().len() > PROFILE_MAX_RANK { return Err(TensorFrameError::ResourceLimit); }
    let frame = inspect_tensor_frame(value.saved_bytes())?;
    validate_tensor_elements(&frame)?;
    if frame.element_type_bytes() != value.element_identity()
        || !frame.shape().eq(value.shape().iter().copied())
        || frame.element_count() != value.elements().len() as u64 {
        return Err(TensorFrameError::Integrity);
    }
    for (bytes, expected) in frame.elements().zip(value.elements()) {
        if decode_element(bytes)? != *expected { return Err(TensorFrameError::Integrity); }
    }
    Ok(())
}

/// The existing ResourceValue canonical scalar owner is the only producer.
/// A borrowed scalar frame is checked before allocation; no host float or cast.
fn decode_element(bytes: &[u8]) -> Result<Fixed64, TensorFrameError> {
    // F5-GA[TENSOR_SCALAR]: Short-circuit len!=9 rejects before tag indexing; the remaining tail then has exactly eight bytes.
    if bytes.len() != 9 || bytes[0] != 0x02 {
        return Err(TensorFrameError::ElementValue);
    }
    let value = Fixed64::from_raw_i64(i64::from_le_bytes(bytes[1..].try_into().unwrap()));
    // Every eight-byte little-endian pattern is an exact Fixed64 raw value.
    // The fixed tag and length above are its complete canonical admission;
    // do not allocate a re-encoding buffer before all elements are validated.
    Ok(value)
}

fn validate_tensor_elements(candidate: &UnadmittedTensorFrame<'_>) -> Result<(), TensorFrameError> {
    element_identity::validate_bytes(candidate.element_type_bytes())
        .map_err(TensorFrameError::ElementType)?;
    for bytes in candidate.elements() { decode_element(bytes)?; }
    Ok(())
}

/// Complete typed admission precedes construction of the immutable value.
pub(super) fn admit_tensor_value(
    saved: &[u8],
    expected_length: usize,
    carrier_hash: &[u8; 32],
    saved_hash: &[u8; 32],
) -> Result<AdmittedTensor, TensorFrameError> {
    let candidate = inspect_sealed_tensor_frame(saved, expected_length, carrier_hash, saved_hash)?;
    if candidate.rank() as usize > PROFILE_MAX_RANK { return Err(TensorFrameError::ResourceLimit); }
    validate_tensor_elements(&candidate)?;
    let mut shape = Vec::new();
    shape.try_reserve_exact(candidate.rank() as usize).map_err(|_| TensorFrameError::Capacity)?;
    shape.extend(candidate.shape());
    let mut elements = Vec::new();
    let count = usize::try_from(candidate.element_count()).map_err(|_| TensorFrameError::Capacity)?;
    elements.try_reserve_exact(count).map_err(|_| TensorFrameError::Capacity)?;
    for bytes in candidate.elements() { elements.push(decode_element(bytes)?); }
    let mut owned = Vec::new();
    owned.try_reserve_exact(saved.len()).map_err(|_| TensorFrameError::Capacity)?;
    owned.extend_from_slice(saved);
    Ok(AdmittedTensor { shape, elements, saved: owned })
}

pub(super) fn encode_typed_tensor(
    resolved: &TypeRef, shape: &[u64], elements: &[Fixed64],
) -> Result<AdmittedTensor, TensorFrameError> {
    // F5-GA[TENSOR_ENCODER]: Successful encode_tensor_candidate has pushed the leading tag before any caller takes saved[1..].
    if shape.len() > PROFILE_MAX_RANK { return Err(TensorFrameError::ResourceLimit); }
    let identity = element_identity::issue(resolved).map_err(TensorFrameError::ElementType)?;
    if shape_count(shape.iter().copied())? != elements.len() as u64 {
        return Err(TensorFrameError::Count);
    }
    let mut payloads = Vec::new();
    payloads.try_reserve_exact(elements.len()).map_err(|_| TensorFrameError::Capacity)?;
    for value in elements {
        let mut bytes = Vec::new();
        ResourceValue::Fixed64(*value).encode_canon(&mut bytes);
        payloads.push(bytes);
    }
    let refs: Vec<&[u8]> = payloads.iter().map(Vec::as_slice).collect();
    let saved = encode_tensor_candidate(&identity, shape, &refs)?;
    let carrier_hash = Sha256::digest(&saved[1..]).into();
    let saved_hash = Sha256::digest(&saved).into();
    admit_tensor_value(&saved, saved.len(), &carrier_hash, &saved_hash)
}

/// RD-510 explicit conversion: lists are input data, never Tensor identity.
/// Validate the entire tree and checked count before allocating flat payload.
pub(super) fn admit_collection(
    resolved: &TypeRef, shape: &[u64], input: &ddonirang_lang::runtime::Value,
) -> Result<AdmittedTensor, TensorFrameError> {
    if shape.len() > PROFILE_MAX_RANK { return Err(TensorFrameError::ResourceLimit); }
    use ddonirang_lang::runtime::Value;
    let identity = element_identity::issue(resolved).map_err(TensorFrameError::ElementType)?;
    element_identity::verify(&identity, resolved).map_err(TensorFrameError::ElementType)?;
    let count = shape_count(shape.iter().copied())?;
    u32::try_from(shape.len()).map_err(|_| TensorFrameError::Rank)?;
    u32::try_from(count).map_err(|_| TensorFrameError::Count)?;
    fn inspect(value: &Value, shape: &[u64]) -> Result<(), TensorFrameError> {
        match shape.split_first() {
            None => match value {
                Value::Fixed64(_) => Ok(()),
                _ => Err(TensorFrameError::ElementValue),
            },
            Some((axis, rest)) => match value {
                Value::List(items) if items.len() as u64 == *axis => {
                    for item in items { inspect(item, rest)?; }
                    Ok(())
                }
                _ => Err(TensorFrameError::ShapeMismatch),
            },
        }
    }
    inspect(input, shape)?;
    let mut elements = Vec::new();
    elements.try_reserve_exact(usize::try_from(count).map_err(|_| TensorFrameError::Capacity)?)
        .map_err(|_| TensorFrameError::Capacity)?;
    fn flatten(value: &Value, elements: &mut Vec<Fixed64>) {
        match value {
            Value::Fixed64(value) => elements.push(*value),
            Value::List(items) => for item in items { flatten(item, elements); },
            _ => unreachable!("entire input admitted before allocation"),
        }
    }
    flatten(input, &mut elements);
    encode_typed_tensor(resolved, shape, &elements)
}

/// Canonical candidate assembly only. The borrowed inputs must subsequently be
/// admitted by the existing type/value owner before any runtime publication.
pub(super) fn encode_tensor_candidate(
    element_type: &[u8],
    shape: &[u64],
    elements: &[&[u8]],
) -> Result<Vec<u8>, TensorFrameError> {
    if element_type.is_empty() {
        return Err(TensorFrameError::EmptyType);
    }
    let rank = u32::try_from(shape.len()).map_err(|_| TensorFrameError::Rank)?;
    let count = u32::try_from(elements.len()).map_err(|_| TensorFrameError::Count)?;
    if shape_count(shape.iter().copied())? != u64::from(count) {
        return Err(TensorFrameError::Count);
    }
    let shape_length = shape
        .len()
        .checked_mul(8)
        .and_then(|n| n.checked_add(4))
        .ok_or(TensorFrameError::Length)?;
    let payload_length = elements.iter().try_fold(4usize, |total, bytes| {
        if bytes.is_empty() {
            return Err(TensorFrameError::EmptyElement);
        }
        u32::try_from(bytes.len()).map_err(|_| TensorFrameError::Length)?;
        total
            .checked_add(4)
            .and_then(|n| n.checked_add(bytes.len()))
            .ok_or(TensorFrameError::Length)
    })?;
    let lengths = [
        FAMILY.len(),
        2,
        element_type.len(),
        4,
        shape_length,
        8,
        payload_length,
    ];
    let total = lengths
        .iter()
        .try_fold(1 + PROFILE.len() + 3, |total: usize, &length| {
            u32::try_from(length).map_err(|_| TensorFrameError::Length)?;
            total
                .checked_add(5)
                .and_then(|n| n.checked_add(length))
                .ok_or(TensorFrameError::Length)
        })?;
    let mut out = Vec::new();
    out.try_reserve_exact(total)
        .map_err(|_| TensorFrameError::Capacity)?;
    out.push(TAG);
    out.extend_from_slice(PROFILE);
    out.extend_from_slice(&[0, 1, 7]);
    fn header(out: &mut Vec<u8>, tag: u8, length: usize) {
        out.push(tag);
        out.extend_from_slice(&(length as u32).to_be_bytes()); // preflight above
    }
    header(&mut out, 1, FAMILY.len());
    out.extend_from_slice(FAMILY);
    header(&mut out, 2, 2);
    out.extend_from_slice(&1u16.to_be_bytes());
    header(&mut out, 3, element_type.len());
    out.extend_from_slice(element_type);
    header(&mut out, 4, 4);
    out.extend_from_slice(&rank.to_be_bytes());
    header(&mut out, 5, shape_length);
    out.extend_from_slice(&rank.to_be_bytes());
    for axis in shape {
        out.extend_from_slice(&axis.to_be_bytes());
    }
    header(&mut out, 6, 8);
    out.extend_from_slice(&u64::from(count).to_be_bytes());
    header(&mut out, 7, payload_length);
    out.extend_from_slice(&count.to_be_bytes());
    for element in elements {
        out.extend_from_slice(&(element.len() as u32).to_be_bytes());
        out.extend_from_slice(element);
    }
    debug_assert_eq!(out.len(), total);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha3_p6_tensor_profile_rank_ceiling_precedes_materialization() {
        let kind = TypeRef::Named("셈수".into());
        assert!(encode_typed_tensor(&kind,&vec![0; PROFILE_MAX_RANK],&[]).is_ok());
        assert_eq!(encode_typed_tensor(&kind,&vec![0; PROFILE_MAX_RANK+1],&[]),
            Err(TensorFrameError::ResourceLimit));
        assert_eq!(admit_collection(&kind,&vec![0; PROFILE_MAX_RANK+1],
            &ddonirang_lang::runtime::Value::List(vec![])), Err(TensorFrameError::ResourceLimit));
        // The structural carrier remains meaningful; this product profile
        // explicitly refuses it rather than redefining its type or truncating axes.
        let identity = element_identity::issue(&kind).unwrap();
        let saved = encode_tensor_candidate(&identity,&vec![0;PROFILE_MAX_RANK+1],&[]).unwrap();
        assert!(inspect_tensor_frame(&saved).is_ok());
        let carrier_hash = Sha256::digest(&saved[1..]).into();
        let saved_hash = Sha256::digest(&saved).into();
        assert_eq!(admit_tensor_value(&saved,saved.len(),&carrier_hash,&saved_hash),
            Err(TensorFrameError::ResourceLimit));
        assert!(preflight_collection(&[0,u64::MAX]).is_ok());
        assert_eq!(preflight_collection(&[u64::MAX,0]), Err(TensorFrameError::ResourceLimit));
        assert!(preflight_collection(&[999_999,0]).is_ok());
        assert_eq!(preflight_collection(&[1_000_000,0]), Err(TensorFrameError::ResourceLimit));
    }

    #[test]
    fn vc14_collection_runtime_admission_and_tamper() {
        use ddonirang_lang::runtime::{Value, TensorValue};
        let kind = TypeRef::Named("셈수".into());
        let scalar = Value::Fixed64(Fixed64::from_raw_i64(17));
        let input = Value::List(vec![scalar.clone(), scalar.clone()]);
        let tensor = admit_collection(&kind, &[2], &input).unwrap().into_runtime().unwrap();
        verify_runtime(&tensor).unwrap();
        let forged = TensorValue::from_admitted_parts(vec![0], tensor.shape().to_vec(), tensor.elements().to_vec(), tensor.saved_bytes().to_vec());
        assert_eq!(verify_runtime(&forged), Err(TensorFrameError::Integrity));
        assert!(admit_collection(&TypeRef::Named("글".into()), &[2], &input).is_err());
        assert!(admit_collection(&kind, &[3], &input).is_err());
        assert!(admit_collection(&kind, &[u64::MAX, 2], &input).is_err());
        assert!(admit_collection(&kind, &[2, 1], &input).is_err());
        assert!(admit_collection(&kind, &[], &input).is_err());
        assert!(admit_collection(&kind, &[], &scalar).is_ok());
        assert!(admit_collection(&kind, &[0], &Value::List(vec![])).is_ok());
        let ragged = Value::List(vec![Value::List(vec![scalar.clone()]), Value::List(vec![scalar.clone(), scalar.clone(), scalar])]);
        assert!(admit_collection(&kind, &[2, 2], &ragged).is_err());
        let mut damaged = tensor.saved_bytes().to_vec();
        damaged.push(0);
        let forged = TensorValue::from_admitted_parts(tensor.element_identity().to_vec(), tensor.shape().to_vec(), tensor.elements().to_vec(), damaged);
        assert!(verify_runtime(&forged).is_err());
    }

    const RANK_TWO: &str = "0e44444e2d42415349432d54454e534f522d56414c55452d563100000107010000001064646e2e74656e736f722e6261736963020000000200010300000008545950452d49363404000000040000000205000000140000000200000000000000020000000000000002060000000800000000000000040700000018000000040000000101000000010200000001030000000104";
    const ZERO_AXIS: &str = "0e44444e2d42415349432d54454e534f522d56414c55452d563100000107010000001064646e2e74656e736f722e6261736963020000000200010300000008545950452d493634040000000400000001050000000c00000001000000000000000006000000080000000000000000070000000400000000";
    const RANK_ZERO: &str = "0e44444e2d42415349432d54454e534f522d56414c55452d563100000107010000001064646e2e74656e736f722e6261736963020000000200010300000008545950452d493634040000000400000000050000000400000000060000000800000000000000010700000009000000010000000101";
    const BAD_RANK_ZERO: &str = "0e44444e2d42415349432d54454e534f522d56414c55452d563100000107010000001064646e2e74656e736f722e6261736963020000000200010300000008545950452d49363404000000040000000005000000040000000006000000080000000000000000070000000400000000";
    fn hash(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    #[test]
    fn opaque_historical_type_is_not_admitted_by_rd682() {
        for vector in [RANK_TWO, ZERO_AXIS, RANK_ZERO] {
            let saved = hex::decode(vector).unwrap();
            assert!(inspect_tensor_frame(&saved).is_ok());
            assert_eq!(
                admit_tensor_value(&saved, saved.len(), &hash(&saved[1..]), &hash(&saved)),
                Err(TensorFrameError::ElementType(element_identity::ElementTypeError::LimitExceeded))
            );
        }
        let saved = hex::decode(RANK_TWO).unwrap();
        assert_eq!(
            admit_tensor_value(&saved, saved.len(), &hash(&saved), &hash(&saved[1..])),
            Err(TensorFrameError::Integrity)
        );
    }
    #[test]
    fn typed_saved_restore_preserves_shape_raw_values_and_bytes() {
        let kind = TypeRef::Named("셈수".into());
        for (shape, raw) in [
            (vec![], vec![i64::MIN]),
            (vec![0], vec![]),
            (vec![2, 2], vec![i64::MIN, -1, 0, i64::MAX]),
        ] {
            let values: Vec<_> = raw.iter().copied().map(Fixed64::from_raw_i64).collect();
            let fixed = encode_typed_tensor(&kind, &shape, &values).unwrap();
            assert_eq!(fixed.shape(), shape);
            assert_eq!(fixed.elements(), values);
            assert_eq!(fixed.saved_bytes()[0], 0x0e);
            let mut restored = fixed.clone();
            for _ in 0..2 {
                let bytes = restored.saved_bytes();
                restored = admit_tensor_value(bytes, bytes.len(), &hash(&bytes[1..]), &hash(bytes)).unwrap();
                assert_eq!(restored, fixed);
                assert_eq!(encode_typed_tensor(&kind, restored.shape(), restored.elements()).unwrap(), fixed);
            }
        }
    }

    #[test]
    fn same_shape_replacement_preserves_old_complete_on_all_rejections() {
        let kind = TypeRef::Named("셈수".into());
        let old = encode_typed_tensor(&kind, &[2, 3], &[Fixed64::from_raw_i64(7); 6]).unwrap();
        let next = encode_typed_tensor(&kind, &[2, 3], &[Fixed64::from_raw_i64(-8); 6]).unwrap();
        let bytes = next.saved_bytes();
        assert_eq!(old.prepare_replacement(bytes, &hash(&bytes[1..]), &hash(bytes)), Ok(next.clone()));
        let changed_shape = encode_typed_tensor(&kind, &[3, 2], next.elements()).unwrap();
        let bytes = changed_shape.saved_bytes();
        assert_eq!(old.prepare_replacement(bytes, &hash(&bytes[1..]), &hash(bytes)), Err(TensorFrameError::ShapeMismatch));
        let preserved = old.clone();
        for length in 0..next.saved_bytes().len() {
            let bytes = &next.saved_bytes()[..length];
            let carrier = bytes.get(1..).unwrap_or_default();
            assert!(old.prepare_replacement(bytes, &hash(carrier), &hash(bytes)).is_err());
            assert_eq!(old, preserved);
        }
        let mut suffix = next.saved_bytes().to_vec();
        suffix.push(0);
        assert!(old.prepare_replacement(&suffix, &hash(&suffix[1..]), &hash(&suffix)).is_err());
        let mut foreign = next.saved_bytes().to_vec();
        foreign[0] = 0x0a;
        assert!(old.prepare_replacement(&foreign, &hash(&foreign[1..]), &hash(&foreign)).is_err());
        assert_eq!(old, preserved);
    }

    #[test]
    fn invalid_scalar_payload_and_foreign_type_never_replace_old_complete() {
        let kind = TypeRef::Named("셈수".into());
        let old = encode_typed_tensor(&kind, &[1], &[Fixed64::from_raw_i64(17)]).unwrap();
        let identity = element_identity::issue(&kind).unwrap();
        let valid = [2, 17, 0, 0, 0, 0, 0, 0, 0];
        for bad in [vec![0], vec![1, 1], vec![2, 0], vec![2; 10], vec![3; 9]] {
            let saved = encode_tensor_candidate(&identity, &[2], &[&valid, &bad]).unwrap();
            let mut published = old.clone();
            let candidate = admit_tensor_value(&saved, saved.len(), &hash(&saved[1..]), &hash(&saved));
            assert_eq!(candidate, Err(TensorFrameError::ElementValue));
            if let Ok(next) = candidate { published = next; }
            assert_eq!(published, old);
        }
        let mut foreign = identity;
        foreign[50] = 2;
        let saved = encode_tensor_candidate(&foreign, &[1], &[&valid]).unwrap();
        assert!(admit_tensor_value(&saved, saved.len(), &hash(&saved[1..]), &hash(&saved)).is_err());
        assert!(encode_typed_tensor(&TypeRef::Named("바른수".into()), &[1], &[Fixed64::from_raw_i64(1)]).is_err());
    }

    #[test]
    fn rank_zero_is_not_scalar_or_rank_one_and_axis_order_is_identity() {
        let kind = TypeRef::Named("셈수".into());
        let one = Fixed64::from_raw_i64(1);
        let zero = encode_typed_tensor(&kind, &[], &[one]).unwrap();
        let vector = encode_typed_tensor(&kind, &[1], &[one]).unwrap();
        let mut scalar = Vec::new();
        ResourceValue::Fixed64(one).encode_canon(&mut scalar);
        assert_ne!(zero.saved_bytes(), scalar);
        assert_ne!(zero, vector);
        let items = [one; 6];
        assert_ne!(encode_typed_tensor(&kind, &[2, 3], &items).unwrap(), encode_typed_tensor(&kind, &[3, 2], &items).unwrap());
        assert!(encode_typed_tensor(&kind, &[], &[]).is_err());
        assert!(encode_typed_tensor(&kind, &[u64::MAX, 2], &[]).is_err());
    }

    fn field_offsets(bytes: &[u8]) -> Vec<usize> {
        let mut offset = 1 + PROFILE.len() + 3;
        (0..7)
            .map(|_| {
                let start = offset;
                let length =
                    u32::from_be_bytes(bytes[offset + 1..offset + 5].try_into().unwrap()) as usize;
                offset += 5 + length;
                start
            })
            .collect()
    }

    #[test]
    fn fixed_packet_vectors_are_exact_not_runtime_types() {
        for (hex_bytes, length, carrier_sha, saved_sha) in [
            (
                RANK_TWO,
                147,
                "a27ac9ece081c0ac8e7719a6efbf1558377f07854e7c00cdf08dca8cba202140",
                "43654153db4ae5edc9d6a53e93ad633d8ab6836a4efe1db6cac02a94f2a56299",
            ),
            (
                ZERO_AXIS,
                119,
                "6f2fa593b382e551349434bad3a389cec2e217cb718c0624fe73fbb5f9e5d9fa",
                "0c82410cb1df57ffa03b7eacb5bd364ae374e39519bc9e49f96d3a22e3099f9b",
            ),
            (
                RANK_ZERO,
                116,
                "7569fbe98fb832eaf376adcdf84fe56f833f0a1b8d49fdcc9d1a3419519765a7",
                "c0b691dc531b93d8608d1eab026b606dd2b805009956182e35babb27c813a194",
            ),
        ] {
            let saved = hex::decode(hex_bytes).unwrap();
            assert_eq!(saved.len(), length);
            assert_eq!(hex::encode(hash(&saved[1..])), carrier_sha);
            assert_eq!(hex::encode(hash(&saved)), saved_sha);
            let frame =
                inspect_sealed_tensor_frame(&saved, length, &hash(&saved[1..]), &hash(&saved))
                    .unwrap();
            assert_eq!(frame.element_type_bytes(), b"TYPE-I64"); // opaque test-only identity
            let shape: Vec<_> = frame.shape().collect();
            let elements: Vec<_> = frame.elements().collect();
            assert_eq!(frame.rank() as usize, shape.len());
            assert_eq!(frame.element_count() as usize, elements.len());
            assert_eq!(
                frame.saved_bytes(),
                encode_tensor_candidate(frame.element_type_bytes(), &shape, &elements).unwrap()
            );
        }
        let bad = hex::decode(BAD_RANK_ZERO).unwrap();
        assert_eq!(
            hex::encode(hash(&bad)),
            "d5688a8f709f5e12f552221d7db3274ba2226520bb9a66e9366502d5263d463a"
        );
        assert_eq!(inspect_tensor_frame(&bad), Err(TensorFrameError::Count));
    }

    #[test]
    fn rank_zero_requires_one_element_and_is_not_scalar() {
        assert_eq!(
            encode_tensor_candidate(b"type", &[], &[]),
            Err(TensorFrameError::Count)
        );
        let saved = encode_tensor_candidate(b"type", &[], &[&[1]]).unwrap();
        assert_eq!(inspect_tensor_frame(&saved).unwrap().element_count(), 1);
        assert_eq!(inspect_tensor_frame(&[1]), Err(TensorFrameError::Tag));
    }

    #[test]
    fn zero_axis_is_order_independent_and_overflow_is_checked() {
        assert_eq!(shape_count([u64::MAX, 2, 0].into_iter()), Ok(0));
        assert_eq!(shape_count([0, u64::MAX, 2].into_iter()), Ok(0));
        assert_eq!(
            shape_count([u64::MAX, 2].into_iter()),
            Err(TensorFrameError::ShapeOverflow)
        );
        assert!(encode_tensor_candidate(b"type", &[u64::MAX, 2, 0], &[]).is_ok());
    }

    #[test]
    fn tag_profile_versions_and_field_count_fail_closed() {
        let base = hex::decode(RANK_TWO).unwrap();
        for tag in (0..=13).chain([15, 255]) {
            let mut wrong = base.clone();
            wrong[0] = tag;
            assert_eq!(inspect_tensor_frame(&wrong), Err(TensorFrameError::Tag));
        }
        for (offset, error) in [
            (1, TensorFrameError::Profile),
            (PROFILE.len() + 2, TensorFrameError::Version),
            (PROFILE.len() + 3, TensorFrameError::Fields),
        ] {
            let mut wrong = base.clone();
            wrong[offset] ^= 1;
            assert_eq!(inspect_tensor_frame(&wrong), Err(error));
        }
        let mut wrong = base.clone();
        wrong[field_offsets(&base)[1] + 6] = 2;
        assert_eq!(inspect_tensor_frame(&wrong), Err(TensorFrameError::Version));
    }

    #[test]
    fn duplicate_unknown_reordered_and_missing_fields_fail_closed() {
        let base = hex::decode(RANK_TWO).unwrap();
        for (index, offset) in field_offsets(&base).into_iter().enumerate() {
            for wrong_tag in [0, 8, index as u8, index as u8 + 2] {
                let mut wrong = base.clone();
                wrong[offset] = wrong_tag;
                assert_eq!(inspect_tensor_frame(&wrong), Err(TensorFrameError::Fields));
            }
        }
    }

    #[test]
    fn every_truncation_and_trailing_byte_is_rejected_without_allocation() {
        let base = hex::decode(RANK_TWO).unwrap();
        for end in 0..base.len() {
            assert!(inspect_tensor_frame(&base[..end]).is_err(), "{end}");
        }
        let mut trailing = base.clone();
        trailing.push(0);
        assert_eq!(
            inspect_tensor_frame(&trailing),
            Err(TensorFrameError::Length)
        );
        for offset in field_offsets(&base) {
            let mut wrong = base.clone();
            wrong[offset + 1..offset + 5].fill(255);
            assert!(inspect_tensor_frame(&wrong).is_err());
        }
    }

    #[test]
    fn rank_shape_and_two_element_counts_are_cross_checked() {
        let base = hex::decode(RANK_TWO).unwrap();
        let offsets = field_offsets(&base);
        for (offset, error) in [
            (offsets[3] + 8, TensorFrameError::Rank),
            (offsets[4] + 8, TensorFrameError::Rank),
            (offsets[5] + 12, TensorFrameError::Count),
            (offsets[6] + 8, TensorFrameError::Count),
        ] {
            let mut wrong = base.clone();
            wrong[offset] ^= 1;
            assert_eq!(inspect_tensor_frame(&wrong), Err(error));
        }
    }

    #[test]
    fn hash_domains_and_sealed_length_cannot_be_substituted() {
        let base = hex::decode(RANK_TWO).unwrap();
        let carrier = hash(&base[1..]);
        let saved = hash(&base);
        assert_eq!(
            inspect_sealed_tensor_frame(&base, base.len(), &saved, &carrier),
            Err(TensorFrameError::Integrity)
        );
        assert_eq!(
            inspect_sealed_tensor_frame(&base, base.len() + 1, &carrier, &saved),
            Err(TensorFrameError::Length)
        );
        let mut changed = base.clone();
        *changed.last_mut().unwrap() ^= 1;
        assert_eq!(
            inspect_sealed_tensor_frame(&changed, changed.len(), &carrier, &saved),
            Err(TensorFrameError::Integrity)
        );
    }

    #[test]
    fn identity_axis_order_and_payload_order_are_not_inferred() {
        let elements: &[&[u8]] = &[&[1], &[2], &[3], &[4], &[5], &[6]];
        let base = encode_tensor_candidate(b"type-a", &[2, 3], elements).unwrap();
        assert_ne!(
            base,
            encode_tensor_candidate(b"type-b", &[2, 3], elements).unwrap()
        );
        assert_ne!(
            base,
            encode_tensor_candidate(b"type-a", &[3, 2], elements).unwrap()
        );
        let mut reversed = elements.to_vec();
        reversed.reverse();
        assert_ne!(
            base,
            encode_tensor_candidate(b"type-a", &[2, 3], &reversed).unwrap()
        );
        let frame = inspect_tensor_frame(&base).unwrap();
        assert_eq!(frame.elements().collect::<Vec<_>>(), elements);
        for _ in 0..2 {
            assert_eq!(
                base,
                encode_tensor_candidate(b"type-a", &[2, 3], elements).unwrap()
            );
        }
    }

    #[test]
    fn empty_type_element_and_shape_mismatch_never_produce_candidate() {
        assert_eq!(
            encode_tensor_candidate(b"", &[1], &[&[1]]),
            Err(TensorFrameError::EmptyType)
        );
        assert_eq!(
            encode_tensor_candidate(b"type", &[1], &[&[]]),
            Err(TensorFrameError::EmptyElement)
        );
        assert_eq!(
            encode_tensor_candidate(b"type", &[2], &[&[1]]),
            Err(TensorFrameError::Count)
        );
    }
}
