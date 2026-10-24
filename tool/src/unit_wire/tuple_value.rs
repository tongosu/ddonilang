//! Approved private tuple_v1 transport. This checks bytes, not source custody.
use ddonirang_core::platform::ResourceValue;
use serde_json::Value;
use sha2::{Digest, Sha256};
pub(super) const FIELDS: [&str; 6] = ["canonical_base64url", "canonical_byte_length", "canonical_sha256", "resource_value_tag", "type", "version"];
const MAX_BYTES: usize = 2_000_000;
fn invalid() -> String { "E_TUPLE_PERSISTENCE_UNBOUND: 위치형 저장 바이트·형식·씨를 검증할 수 없습니다".into() }

pub(super) fn candidate(payload: &Value) -> Result<ResourceValue, String> {
    candidate_with_budget(payload, &mut super::saved_resource_value::ValueDecodeBudget::new(),&[],None,None)
}

pub(super) fn candidate_with_budget(payload: &Value, budget: &mut super::saved_resource_value::ValueDecodeBudget,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>) -> Result<ResourceValue, String> {
    let obj = payload.as_object().ok_or_else(invalid)?;
    let rational = obj.get("type").and_then(Value::as_str) == Some("rational_v1");
    if obj.len() != FIELDS.len() || !obj.keys().map(String::as_str).eq(FIELDS)
        || obj["type"].as_str() != Some(if rational {"rational_v1"} else {"tuple_v1"})
        || obj["resource_value_tag"].as_str() != Some(if rational {"10"} else {"0f"})
        || obj["version"].as_u64() != Some(1) { return Err(invalid()); }
    let length = obj["canonical_byte_length"].as_u64().and_then(|n| usize::try_from(n).ok()).ok_or_else(invalid)?;
    if !(19..=MAX_BYTES).contains(&length) { return Err(invalid()); }
    let encoded = obj["canonical_base64url"].as_str().ok_or_else(invalid)?;
    let expected = length.checked_mul(4).and_then(|n| n.checked_add(2)).map(|n| n / 3).ok_or_else(invalid)?;
    if encoded.len() != expected || encoded.len() % 4 == 1 { return Err(invalid()); }
    let hash = obj["canonical_sha256"].as_str().ok_or_else(invalid)?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) { return Err(invalid()); }
    // Validate the complete alphabet and unused bits before allocating bytes.
    fn digit(byte: u8) -> Option<u8> { match byte {
        b'A'..=b'Z' => Some(byte-b'A'), b'a'..=b'z' => Some(byte-b'a'+26), b'0'..=b'9' => Some(byte-b'0'+52), b'-' => Some(62), b'_' => Some(63), _ => None,
    } }
    if !encoded.bytes().all(|b| digit(b).is_some()) { return Err(invalid()); }
    let last = digit(*encoded.as_bytes().last().ok_or_else(invalid)?).ok_or_else(invalid)?;
    if (encoded.len()%4==2 && last&15!=0) || (encoded.len()%4==3 && last&3!=0) { return Err(invalid()); }
    let mut bytes = Vec::new(); bytes.try_reserve_exact(length).map_err(|_| invalid())?;
    let (mut bits, mut count) = (0u32, 0u32);
    for byte in encoded.bytes() {
        bits = (bits << 6) | u32::from(digit(byte).ok_or_else(invalid)?); count += 6;
        if count >= 8 { count -= 8; bytes.push((bits >> count) as u8); bits &= (1 << count) - 1; }
    }
    if bytes.len() != length || hex::encode(Sha256::digest(&bytes)) != hash { return Err(invalid()); }
    let value = super::saved_resource_value::decode_saved_resource_value_with_catalog(&bytes,budget,contracts,references,catalog,false).map_err(|_| invalid())?;
    if if rational {!matches!(&value, ResourceValue::RationalV1(_))} else {!matches!(&value, ResourceValue::Tuple { .. })} { return Err(invalid()); }
    if super::resource_value_to_detjson(&value) != *payload { return Err(invalid()); }
    Ok(value)
}

pub(crate) fn contains_tuple(root: &ResourceValue) -> bool {
    contains_source_bound_value(root, false)
}

pub(crate) fn requires_source_type(root: &ResourceValue) -> bool {
    contains_source_bound_value(root, true)
}

fn contains_source_bound_value(root: &ResourceValue, rational:bool) -> bool {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() { match value {
        ResourceValue::Tuple { .. } => return true,
        ResourceValue::RationalV1(_) if rational => return true,
        ResourceValue::List(items) => pending.extend(items.iter()),
        ResourceValue::Set(items) => pending.extend(items.values()),
        ResourceValue::Record { fields, .. } => pending.extend(fields.values()),
        ResourceValue::Map(items) => for item in items.values() { pending.push(&item.key); pending.push(&item.value); },
        _ => {},
    } }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::platform::RecordFieldType as R;

    #[test]
    fn n1_rational_transport_is_strict_and_source_less_candidate_is_not_admitted() {
        let numeric=ddonirang_core::saved_rational::NormalizedRationalValueV1::from_parts(1.into(),3.into()).unwrap();
        let value=ResourceValue::RationalV1(numeric);
        let det=super::super::resource_value_to_detjson(&value);
        assert_eq!(det["resource_value_tag"],"10");assert_eq!(det["type"],"rational_v1");
        let raw=det.to_string();
        let outer=serde_json::json!({"value_det":{"정확":det.clone()}});
        let mut pretty=Vec::new();super::super::write_pretty_json(&mut pretty,&outer).unwrap();
        let pretty=String::from_utf8(pretty).unwrap();assert!(pretty.contains(&raw));
        assert_eq!(*super::super::parse_snapshot_json(&pretty).unwrap(),outer);
        let parsed=super::super::parse_snapshot_json(&raw).unwrap();
        assert_eq!(candidate(&parsed).unwrap(),value);
        assert!(super::super::resource_value_from_detjson(&parsed).is_err(),"no source/type custody from bytes alone");
        for (field,bad) in [("version",serde_json::json!("1")),("version",serde_json::json!(1.0)),
            ("version",serde_json::json!(true)),("resource_value_tag",serde_json::json!("0f")),
            ("canonical_sha256",serde_json::json!("7022E00436F7ED8AA070FBCC7C8F1DFD32A02A96A02A6B152D1A371D2E3D7FA6"))] {
            let mut wrong=det.clone();wrong[field]=bad;assert!(candidate(&wrong).is_err(),"{field}");
        }
        let reversed=raw.replacen("{\"canonical_base64url\":","{\"unknown\":0,\"canonical_base64url\":",1);
        assert!(super::super::parse_snapshot_json(&reversed).is_err());
        assert!(super::super::parse_snapshot_json(&(" ".to_owned()+&raw)).is_err());
        let tuple=ResourceValue::Tuple{element_types:vec![R::Named{name:"나눔수".into()}],items:vec![value]};
        let mut bytes=Vec::new();tuple.encode_canon(&mut bytes);
        assert_eq!(super::super::saved_resource_value::decode_saved_resource_value(&bytes).unwrap(),tuple);
    }

    #[test]
    fn vc13_tuple_materialization_budget_is_shared_across_resource_rows() {
        let n = 16_000_000 / std::mem::size_of::<ResourceValue>() / 4;
        let value = ResourceValue::Tuple {
            element_types: vec![R::Applied { name: "차림".into(), args: vec![R::Named { name: "참거짓".into() }] }],
            items: vec![ResourceValue::List(vec![ResourceValue::Bool(true); n])],
        };
        let det = super::super::resource_value_to_detjson(&value);
        let mut snapshot = serde_json::json!({"schema":super::super::RESOURCE_SNAPSHOT_SCHEMA_V2,
            "unit_wire_schema":super::super::UNIT_VALUE_WIRE_SCHEMA_V2,
            "components":[],"next_entity":0,"json":{},"handle":{},"fixed64":{},"value_det":{"a":det}});
        super::super::resource_snapshot::check_resource_snapshot_structure(&snapshot).unwrap();
        snapshot["value_det"]["b"] = snapshot["value_det"]["a"].clone();
        snapshot["value_det"]["c"] = snapshot["value_det"]["a"].clone();
        assert!(snapshot.to_string().len() < MAX_BYTES, "outer byte cap alone cannot prove materialization budget");
        assert!(super::super::resource_snapshot::check_resource_snapshot_structure(&snapshot).is_err(),
            "per-row budget reset must not amplify a legal small carrier into an over-budget world");
    }

    #[test]
    fn vc13_tuple_pretty_output_keeps_only_the_carrier_leaf_compact() {
        let value = ResourceValue::Tuple { element_types: Vec::new(), items: Vec::new() };
        let det = super::super::resource_value_to_detjson(&value);
        let payload = serde_json::json!({"value_det":{"빈묶음":det},"other":[1,2]});
        let mut pretty = Vec::new(); super::super::write_pretty_json(&mut pretty, &payload).unwrap();
        let pretty = String::from_utf8(pretty).unwrap();
        assert!(pretty.contains(&det.to_string()));
        assert_eq!(*super::super::parse_snapshot_json(&pretty).unwrap(), payload);
    }

    #[test]
    fn vc13_tuple_fixed_design_bytes8_negative27_alias_open_and_legacy_bytes5() {
        // Historical proposal headers are retained in this JSON-equivalent
        // UTF-8/LF fixture copy; later Owner assignment authorizes constants.
        // These expectations come from the fixed design, not product output.
        let fixture: Value = serde_json::from_str(include_str!("../../tests/fixtures/vc13_tuple_approved_carrier_vectors.json")).unwrap();
        for row in fixture["positives"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let bytes = hex::decode(row["canonical_hex"].as_str().unwrap()).unwrap();
            assert_eq!(hex::encode(Sha256::digest(&bytes)), row["canonical_sha256"], "{id}");
            let value = super::super::saved_resource_value::decode_saved_resource_value(&bytes).unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let det = super::super::resource_value_to_detjson(&value).to_string();
            assert_eq!(det, row["value_det_utf8"], "{id}");
            let parsed = super::super::parse_snapshot_json(&det).unwrap();
            assert_eq!(candidate(&parsed).unwrap(), value, "{id}");
            // Self-consistent bytes alone must still fail source-less restore.
            assert!(super::super::resource_value_from_detjson(&parsed).is_err(), "{id}");
            let mut patch = ddonirang_core::platform::Patch::default();
            patch.ops.push(ddonirang_core::platform::PatchOp::SetResourceValue { tag: "검사".into(), value });
            let commit = ddonirang_core::commit_canon::canonical_patch_ops_text(&patch);
            assert!(commit.contains(row["commit_utf8"].as_str().unwrap()), "{id}: {commit}");
        }
        let mut negative_count = 0;
        let mut open_alias_count = 0;
        for row in fixture["negative_vectors"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let bytes = hex::decode(row["hex"].as_str().unwrap()).unwrap();
            if id == "alias_type" {
                // The reference model rejects Named 수, but the existing
                // language type owner preserves/adopts it. Do not invent a
                // codec ban or call this unresolved vector a product PASS.
                let current = ddonirang_lang::runtime::record_type_identity(
                    &ddonirang_lang::TypeRef::Named("수".into()), &ddonirang_lang::runtime::RecordSchemas::new()).unwrap();
                assert_eq!(current, R::Named { name: "수".into() });
                open_alias_count += 1;
                continue;
            }
            let rejected = if row["kind"] == "det" {
                super::super::parse_snapshot_json(std::str::from_utf8(&bytes).unwrap()).is_err()
            } else {
                match super::super::saved_resource_value::decode_saved_resource_value(&bytes) {
                    Err(_) => true,
                    Ok(value) if !matches!(&value, ResourceValue::Tuple { .. }) => true,
                    Ok(value) if !row["source_type_expected"].is_null() => {
                        // Explicit fixed source-layout expectation, not inferred
                        // from the swapped value's own declared type bytes.
                        let expected = R::Tuple { items: vec![R::Named { name: "셈수".into() }, R::Named { name: "글".into() }, R::Named { name: "참거짓".into() }] };
                        super::super::saved_resource_value::check_saved_type(&value, &expected).is_err()
                    }
                    Ok(_) => false,
                }
            };
            assert!(rejected, "{id}"); negative_count += 1;
        }
        assert_eq!(negative_count, 27);
        assert_eq!(open_alias_count, 1);
        for row in fixture["compatibility_reference_bytes"].as_array().unwrap() {
            let bytes = hex::decode(row["hex"].as_str().unwrap()).unwrap();
            let value = super::super::saved_resource_value::decode_saved_resource_value(&bytes).unwrap();
            let mut encoded = Vec::new(); value.encode_canon(&mut encoded);
            assert_eq!(bytes, encoded, "{}", row["name"]);
        }
        assert_eq!(fixture["positives"].as_array().unwrap().len(), 8);
        assert_eq!(fixture["compatibility_reference_bytes"].as_array().unwrap().len(), 5);
    }
}
