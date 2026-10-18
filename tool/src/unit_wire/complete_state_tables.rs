//! RD-670 tables: framing is BE; saved ResourceValue bodies retain their own
//! existing codec. A borrowed resource body is not typed/custody admission.
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use ddonirang_core::Fixed64;
use ddonirang_core::platform::ResourceValue;
use std::collections::BTreeMap;

const TOTAL_MAX: usize = 2_000_000;

/// A decoded value plus its recorded declared type, not type admission.
/// The resolved canonical type owner must verify this pair before publication.
#[derive(Debug)]
pub(crate) struct SubjectValueCandidateV1 {
    pub(crate) target_type:Vec<u8>,
    pub(crate) value:ResourceValue,
}

/// Joint saved-value check after all Subject rows have been materialized.
/// Custody alone proves identity/lifecycle, not that the referenced value is
/// present in this candidate. Never recurse through referenced target values,
/// infer its declared type, or consult a live/ambient world here.
pub(super) fn check_reference_value_targets(
    value:&ResourceValue,
    subjects:&BTreeMap<[u8;32],SubjectValueCandidateV1>,
) -> Result<(),Failure> {
    // RD-694 also applies to the joint target check after decoding. Keep the
    // original first-invalid-leaf order without consuming the host call stack.
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            ResourceValue::IdentityReference(reference) => {
                let identity: &[u8; 32] = reference.target_identity.as_slice().try_into()
                    .map_err(|_| Failure::SubjectCustodyInvalid)?;
                let target = subjects.get(identity).ok_or(Failure::SubjectCustodyInvalid)?;
                if target.target_type != reference.target_type_identity {
                    return Err(Failure::SubjectCustodyInvalid);
                }
            }
            ResourceValue::List(values) | ResourceValue::Tuple { items: values, .. } => pending.extend(values.iter().rev()),
            ResourceValue::Set(values) => pending.extend(values.values().rev()),
            ResourceValue::Map(values) => {
                for entry in values.values().rev() {
                    pending.push(&entry.value);
                    pending.push(&entry.key);
                }
            }
            ResourceValue::Record { fields, .. } => pending.extend(fields.values().rev()),
            _ => {}
        }
    }
    Ok(())
}

/// RD-670 section 3.4. Validate the entire ordered table and LIVE custody
/// before decoding any value. Never turn a tombstone into a live value or
/// discard the table merely because no resource currently references it.
pub(super) fn decode_subject_value_table(
    bytes:&[u8],
    custody:Option<&crate::subject_reference_owner::SavedWorldIdentityCustodyV1>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
) -> Result<BTreeMap<[u8;32],SubjectValueCandidateV1>,Failure> {
    check_subject_value_table_rows(bytes)?;
    let references=custody.map(crate::subject_reference_owner::SubjectReferenceContextV1::new).transpose()?;
    decode_subject_value_table_with_custody(bytes,custody,contracts,references.as_ref(),&mut super::saved_resource_value::ValueDecodeBudget::new())
}

pub(super) fn decode_subject_value_table_with_custody(
    bytes:&[u8],custody:Option<&crate::subject_reference_owner::SavedWorldIdentityCustodyV1>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    budget:&mut super::saved_resource_value::ValueDecodeBudget,
) -> Result<BTreeMap<[u8;32],SubjectValueCandidateV1>,Failure> {
    decode_subject_value_table_with_catalog(bytes,custody,contracts,references,budget,None)
}
pub(super) fn decode_subject_value_table_with_catalog(
    bytes:&[u8],custody:Option<&crate::subject_reference_owner::SavedWorldIdentityCustodyV1>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    budget:&mut super::saved_resource_value::ValueDecodeBudget,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
) -> Result<BTreeMap<[u8;32],SubjectValueCandidateV1>,Failure> {
    let count=check_subject_value_table_rows(bytes)?;
    if count!=0&&custody.is_none() {return Err(Failure::SubjectCustodyInvalid);}
    decode_preflighted_subject_values(bytes,count,custody.map(|c|c.rows.as_slice()),contracts,references,budget,catalog)
}

pub(super) fn check_subject_value_table_rows(bytes:&[u8])->Result<u64,Failure>{
    if bytes.len()>TOTAL_MAX {return Err(Failure::LimitExceeded);}
    let mut r=Reader{bytes,at:0};let count=r.u64()?;
    if count>65_536 {return Err(Failure::LimitExceeded);}
    if count>(bytes.len().saturating_sub(8)/41) as u64 {return Err(Failure::InvalidInput);}
    let mut previous:Option<&[u8]>=None;
    for _ in 0..count {
        let identity=r.take(32)?;
        if previous.is_some_and(|old|old>=identity) {return Err(Failure::SubjectCustodyInvalid);}
        previous=Some(identity);
        let length=usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        if length==0 {return Err(Failure::InvalidInput);}
        if length>TOTAL_MAX {return Err(Failure::LimitExceeded);}
        r.take(length)?;
    }
    r.done()?;
    Ok(count)
}

pub(super) fn decode_temporary_subject_values(bytes:&[u8],rows:&[crate::subject_reference_owner::SavedSubjectRow],contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:&crate::subject_reference_owner::SubjectReferenceContextV1<'_>,catalog:&crate::authored_project::registration::SavedNominalCatalogV1)->Result<BTreeMap<[u8;32],SubjectValueCandidateV1>,Failure>{
    let count=check_subject_value_table_rows(bytes)?;
    decode_preflighted_subject_values(bytes,count,Some(rows),contracts,Some(references),&mut super::saved_resource_value::ValueDecodeBudget::new(),Some(catalog))
}
fn decode_preflighted_subject_values(bytes:&[u8],count:u64,custody:Option<&[crate::subject_reference_owner::SavedSubjectRow]>,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,budget:&mut super::saved_resource_value::ValueDecodeBudget,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<BTreeMap<[u8;32],SubjectValueCandidateV1>,Failure>{
    // Custody rows are ordinal ordered, not identity ordered. Build only a
    // bounded borrowed index after ALL row lengths/order/suffix checks.
    let live=match custody {
        Some(saved) if count!=0||catalog.is_some()=>{
            budget.charge(saved.len().checked_mul(128).ok_or(Failure::LimitExceeded)?)?;
            saved.iter().filter(|row|row.live)
                .map(|row|(row.identity,row)).collect::<BTreeMap<_,_>>()
        },
        _=>BTreeMap::new(),
    };
    if catalog.is_some()&&live.len() as u64!=count{return Err(Failure::SubjectCustodyInvalid);}
    // Check every lifecycle binding before constructing any typed value.
    let mut r=Reader{bytes,at:0};r.u64()?;
    for _ in 0..count {
        let identity:&[u8;32]=r.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let row=live.get(identity).ok_or(Failure::SubjectCustodyInvalid)?;
        budget.charge(row.target_type.len().checked_add(128).ok_or(Failure::LimitExceeded)?)?;
        let length=usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;r.take(length)?;
    }
    r.done()?;
    let mut r=Reader{bytes,at:0};r.u64()?;
    let mut values=BTreeMap::new();
    for _ in 0..count {
        let identity:[u8;32]=r.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let length=usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        let value=super::saved_resource_value::decode_saved_resource_value_with_catalog(r.take(length)?,budget,contracts,references,catalog,true)?;
        let row=live.get(&identity).ok_or(Failure::SubjectCustodyInvalid)?;
        if catalog.is_some()&&!matches!(&value,ResourceValue::SubjectPayloadV1(payload) if payload.name_type_identity().as_slice()==row.target_type){return Err(Failure::SubjectCustodyInvalid);}
        values.insert(identity,SubjectValueCandidateV1{target_type:row.target_type.clone(),value});
    }
    r.done()?;Ok(values)
}

/// Existing section-A writer. This is a codec, not a registry issuer or a
/// trusted same-boundary capture. Omission is decided by the execution owner;
/// an empty slice is encoded as count zero, never silently changed to EMPTY.
pub(super) fn encode_registered_atom_evidence(
    contracts: &[crate::registered_atom_contract::RecordedAtomContract],
) -> Result<Vec<u8>, Failure> {
    crate::registered_atom_contract::verify_contract_set(contracts).map_err(|_| Failure::AtomEvidenceInvalid)?;
    let mut length = 8usize;
    let mut previous = None;
    for contract in contracts {
        let custody = contract.custody();
        if previous.is_some_and(|old| old >= custody.snapshot_identity) { return Err(Failure::AtomEvidenceInvalid); }
        previous = Some(custody.snapshot_identity);
        let raw = contract.recorded_bytes();
        if raw.len() > 1_000_000 { return Err(Failure::LimitExceeded); }
        length = length.checked_add(72).and_then(|n| n.checked_add(raw.len())).ok_or(Failure::LimitExceeded)?;
        if length > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| Failure::LimitExceeded)?;
    bytes.extend((contracts.len() as u64).to_be_bytes());
    for contract in contracts {
        let custody = contract.custody();
        bytes.extend(custody.snapshot_identity); bytes.extend(custody.sha256);
        bytes.extend(custody.byte_length.to_be_bytes()); bytes.extend(contract.recorded_bytes());
    }
    Ok(bytes)
}

/// Existing section-B wrapper; zero Subject values do not authorize omission.
pub(super) fn encode_saved_world_custody(
    custody: &crate::subject_reference_owner::SavedWorldIdentityCustodyV1,
) -> Result<Vec<u8>, Failure> {
    use sha2::{Digest, Sha256};
    let length = custody.recorded_byte_length()?.checked_add(40).ok_or(Failure::LimitExceeded)?;
    if length > 1_000_000 { return Err(Failure::LimitExceeded); }
    let raw = custody.encode_recorded()?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| Failure::LimitExceeded)?;
    bytes.extend((raw.len() as u64).to_be_bytes()); bytes.extend(&raw); bytes.extend(Sha256::digest(&raw));
    Ok(bytes)
}

/// RD-670 §3.6 retains the original RD-665 custody bytes and digest.
pub(super) fn decode_saved_world_custody(bytes:&[u8],pesi:&[u8;32])->Result<Option<crate::subject_reference_owner::SavedWorldIdentityCustodyV1>,Failure>{
    if bytes.is_empty(){return Ok(None);}
    let mut r=Reader{bytes,at:0};let length=r.u64()?;
    if length>1_000_000{return Err(Failure::LimitExceeded);}
    let raw=r.take(usize::try_from(length).map_err(|_|Failure::LimitExceeded)?)?;
    let digest=r.take(32)?.try_into().map_err(|_|Failure::SubjectCustodyInvalid)?;
    r.done()?;
    crate::subject_reference_owner::SavedWorldIdentityCustodyV1::load_recorded(raw,length,digest,pesi).map(Some)
}

/// RD-670 §3.5: exact binary projection of RD-668 recorded evidence.
/// Preflight all count/length/order/suffix constraints before atom allocation.
pub(super) fn decode_registered_atom_evidence(bytes:&[u8])->Result<Vec<crate::registered_atom_contract::RecordedAtomContract>,Failure> {
    use crate::registered_atom_contract::{ContractCustody,RecordedAtomContract,verify_contract_set};
    if bytes.is_empty() {return Ok(Vec::new());}
    if bytes.len()>TOTAL_MAX {return Err(Failure::LimitExceeded);}
    let mut r=Reader{bytes,at:0};let count=r.u64()?;
    if count>256 {return Err(Failure::LimitExceeded);}
    if count>(bytes.len().saturating_sub(8)/72) as u64 {return Err(Failure::InvalidInput);}
    let mut previous:Option<&[u8]>=None;
    for _ in 0..count {
        let snapshot=r.take(32)?;
        if previous.is_some_and(|old|old>=snapshot) {return Err(Failure::AtomEvidenceInvalid);}
        previous=Some(snapshot);r.take(32)?;
        let length=r.u64()?;
        if length>1_000_000 {return Err(Failure::LimitExceeded);}
        r.take(usize::try_from(length).map_err(|_|Failure::LimitExceeded)?)?;
    }
    r.done()?;
    let mut r=Reader{bytes,at:0};r.u64()?;
    let mut contracts=Vec::new();
    contracts.try_reserve_exact(count as usize).map_err(|_|Failure::LimitExceeded)?;
    for _ in 0..count {
        let snapshot_identity=r.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let sha256=r.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let byte_length=r.u64()?;
        let raw=r.take(usize::try_from(byte_length).map_err(|_|Failure::LimitExceeded)?)?;
        contracts.push(RecordedAtomContract::restore_recorded(raw,ContractCustody{byte_length,sha256,snapshot_identity})
            .map_err(|_|Failure::AtomEvidenceInvalid)?);
    }
    r.done()?;verify_contract_set(&contracts).map_err(|_|Failure::AtomEvidenceInvalid)?;
    Ok(contracts)
}

struct Reader<'a> { bytes: &'a [u8], at: usize }
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Failure> {
        let end = self.at.checked_add(n).ok_or(Failure::LimitExceeded)?;
        let out = self.bytes.get(self.at..end).ok_or(Failure::InvalidInput)?;
        self.at = end;
        Ok(out)
    }
    fn u64(&mut self) -> Result<u64, Failure> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn name(&mut self) -> Result<&'a str, Failure> {
        let n = u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
        let n = usize::try_from(n).map_err(|_|Failure::LimitExceeded)?;
        std::str::from_utf8(self.take(n)?).map_err(|_|Failure::InvalidInput)
    }
    fn done(&self) -> Result<(), Failure> {
        if self.at == self.bytes.len() { Ok(()) } else { Err(Failure::InvalidInput) }
    }
}

fn reader(bytes: &[u8]) -> Result<(Reader<'_>, u64), Failure> {
    if bytes.len() > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    let mut r = Reader {bytes, at:0};
    let count = r.u64()?;
    // Both row forms have at least a four-byte name length and eight bytes.
    if count > (bytes.len().saturating_sub(8)/12) as u64 { return Err(Failure::InvalidInput); }
    Ok((r,count))
}

fn ordered<'a>(previous: &mut Option<&'a str>, name: &'a str) -> Result<(), Failure> {
    if previous.is_some_and(|p|p.as_bytes() >= name.as_bytes()) { return Err(Failure::InvalidInput); }
    *previous = Some(name);
    Ok(())
}

/// No allocation or value interpretation; caller must admit every value body
/// through the shared typed codec before publishing a restore candidate.
pub(crate) fn check_resource_table_rows(bytes: &[u8]) -> Result<(), Failure> {
    let (mut r,count) = reader(bytes)?;
    let mut previous = None;
    for _ in 0..count {
        let name = r.name()?;
        ordered(&mut previous,name)?;
        let length = usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        if length == 0 { return Err(Failure::InvalidInput); }
        if length > TOTAL_MAX { return Err(Failure::LimitExceeded); }
        r.take(length)?;
    }
    r.done()
}

/// Existing BE table framing and the core value owner, with output allocation
/// preflight. A successful encoding is not source, custody or restore admission.
pub(crate) fn encode_resource_table(values:&BTreeMap<String,ResourceValue>)->Result<Vec<u8>,Failure> {
    encode_resource_table_with_catalog(values,None,None)
}
pub(super) fn encode_resource_table_with_catalog(values:&BTreeMap<String,ResourceValue>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>)->Result<Vec<u8>,Failure>{
    encode_resource_table_with_context(values,catalog,references,&[])
}
pub(super) fn encode_resource_table_with_context(values:&BTreeMap<String,ResourceValue>,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
)->Result<Vec<u8>,Failure>{
    let mut size=8usize;
    let nodes=16_000_000usize/std::mem::size_of::<ResourceValue>();
    let mut lengths=Vec::new();
    if values.len()>nodes{return Err(Failure::LimitExceeded);}
    lengths.try_reserve_exact(values.len()).map_err(|_|Failure::LimitExceeded)?;
    let mut rational_save_budget=super::saved_resource_value::ValueDecodeBudget::new();
    for (name,value) in values {
        rational_save_budget.validate_rational_save(value)?;
        u32::try_from(name.len()).map_err(|_|Failure::LimitExceeded)?;
        size=size.checked_add(12).and_then(|n|n.checked_add(name.len())).ok_or(Failure::LimitExceeded)?;
        let remaining=TOTAL_MAX.checked_sub(size).ok_or(Failure::LimitExceeded)?;
        let length=value.canonical_byte_length(remaining,nodes).ok_or(Failure::LimitExceeded)?;
        size=size.checked_add(length).ok_or(Failure::LimitExceeded)?;
        lengths.push(length);
    }
    let mut bytes=Vec::new();bytes.try_reserve_exact(size).map_err(|_|Failure::LimitExceeded)?;
    bytes.extend_from_slice(&(values.len() as u64).to_be_bytes());
    for ((name,value),length) in values.iter().zip(lengths) {
        bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&(length as u64).to_be_bytes());
        let start=bytes.len();value.encode_canon(&mut bytes);
        if bytes.len()-start!=length{return Err(Failure::InvalidInput);}
    }
    // Includes the shared materialization budget. The caller must provide
    // recorded evidence; the default producer above still supplies none.
    decode_resource_table_with_catalog(&bytes,contracts,references,&mut super::saved_resource_value::ValueDecodeBudget::new(),catalog)?;
    Ok(bytes)
}

/// Shared typed value decoding. Handle/Subject owner and complete-world custody
/// checks are subsequent gates; this map alone is never restore authorization.
pub(crate) fn decode_resource_table(bytes: &[u8]) -> Result<BTreeMap<String,ResourceValue>, Failure> {
    decode_resource_table_with_contracts(bytes,&[])
}

pub(super) fn decode_resource_table_with_contracts(bytes: &[u8],contracts:&[crate::registered_atom_contract::RecordedAtomContract]) -> Result<BTreeMap<String,ResourceValue>, Failure> {
    decode_resource_table_with_custody(bytes,contracts,None,&mut super::saved_resource_value::ValueDecodeBudget::new())
}

pub(super) fn decode_resource_table_with_custody(bytes:&[u8],contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,budget:&mut super::saved_resource_value::ValueDecodeBudget) -> Result<BTreeMap<String,ResourceValue>, Failure> {
    decode_resource_table_with_catalog(bytes,contracts,references,budget,None)
}
pub(super) fn decode_resource_table_with_catalog(bytes:&[u8],contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,budget:&mut super::saved_resource_value::ValueDecodeBudget,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>) -> Result<BTreeMap<String,ResourceValue>, Failure> {
    check_resource_table_rows(bytes)?;
    let (mut r,count)=reader(bytes)?;
    let mut values=BTreeMap::new();
    for _ in 0..count {
        let name=r.name()?;budget.charge(name.len().checked_add(128).ok_or(Failure::LimitExceeded)?)?;
        let name=name.to_owned();
        let length=usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        let value=super::saved_resource_value::decode_saved_resource_value_with_catalog(r.take(length)?,budget,contracts,references,catalog,false)?;
        values.insert(name,value);
    }
    r.done()?;
    Ok(values)
}

pub(crate) fn check_parameter_table(bytes: &[u8]) -> Result<(), Failure> {
    let (mut r,count) = reader(bytes)?;
    let mut previous = None;
    for _ in 0..count {
        let name = r.name()?;
        ordered(&mut previous,name)?;
        r.take(8)?;
    }
    r.done()
}

/// Validate the whole table before materializing; no last-wins or partial map.
pub(crate) fn decode_parameter_table(bytes: &[u8],budget:&mut super::saved_resource_value::ValueDecodeBudget) -> Result<BTreeMap<String,Fixed64>, Failure> {
    check_parameter_table(bytes)?;
    let (mut r,count) = reader(bytes)?;
    let mut values = BTreeMap::new();
    for _ in 0..count {
        let name=r.name()?;budget.charge(name.len().checked_add(128).ok_or(Failure::LimitExceeded)?)?;
        let name=name.to_owned();
        let raw = i64::from_be_bytes(r.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?);
        values.insert(name,Fixed64::from_raw_i64(raw));
    }
    r.done()?;
    Ok(values)
}

pub(crate) fn encode_parameter_table(values: &BTreeMap<String,Fixed64>) -> Result<Vec<u8>, Failure> {
    let mut size = 8usize;
    for name in values.keys() {
        u32::try_from(name.len()).map_err(|_|Failure::LimitExceeded)?;
        size = size.checked_add(12).and_then(|s|s.checked_add(name.len())).ok_or(Failure::LimitExceeded)?;
        if size > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(size).map_err(|_|Failure::LimitExceeded)?;
    bytes.extend_from_slice(&(values.len() as u64).to_be_bytes());
    for (name,value) in values {
        bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&value.raw_i64().to_be_bytes());
    }
    check_parameter_table(&bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_world_custody_section_preserves_zero_reference_nonce_and_allocator() {
        let pesi = [7; 32];
        let owner = crate::subject_reference_owner::LiveSubjectOwnerV1::fresh(pesi).unwrap();
        let section = encode_saved_world_custody(owner.custody()).unwrap();
        assert!(!section.is_empty(), "zero references do not authorize dropping a created world identity");
        let restored = decode_saved_world_custody(&section, &pesi).unwrap().unwrap();
        assert_eq!(restored.encode_recorded().unwrap(), owner.custody().encode_recorded().unwrap());
        let empty = [0; 8];
        let frame = super::super::complete_state_frame::CompleteStateFrameV1 {
            pesi: &pesi, resources: &empty, parameters: &empty, next_tick: 0,
            subjects: &empty, atom_evidence: &[], subject_custody: &section,
        };
        let raw = frame.encode_canonical().unwrap();
        let decoded = super::super::complete_state_frame::check_complete_state_frame_v1(&raw, &pesi).unwrap();
        assert_eq!(decoded.subject_custody, section);
        assert!(decode_saved_world_custody(&section, &[8; 32]).is_err());
        let mut tampered = section.clone();
        tampered[10] ^= 1;
        assert!(decode_saved_world_custody(&tampered, &pesi).is_err());
    }

    #[test]
    fn reference_target_crosscheck_is_stack_safe_for_admitted_saved_trees() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::Bool(true);
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            assert!(check_reference_value_targets(&value, &BTreeMap::new()).is_ok());
            // Exercise the real common frame producer/consumer too, not just
            // the walker: this value fits the unchanged byte/node budgets.
            let resources = encode_resource_table(&BTreeMap::from([("deep".into(), value.clone())])).unwrap();
            let empty = [0; 8];
            let frame = super::super::complete_state_frame::CompleteStateFrameV1 {
                pesi: &[7; 32], resources: &resources, parameters: &empty,
                next_tick: 0, subjects: &empty, atom_evidence: &[], subject_custody: &[],
            };
            let encoded = frame.encode_canonical().unwrap();
            let decoded = super::super::complete_state_frame::check_complete_state_frame_v1(&encoded, &[7; 32]).unwrap();
            assert!(decoded.decode_value_tables().is_ok());
            value.drop_stack_safe();
            let mut value = ResourceValue::IdentityReference(
                ddonirang_core::platform::IdentityReferenceValueV1 {
                    profile_version: 1, target_type_identity: vec![1],
                    owner_space_identity: vec![2; 32], target_identity: vec![3; 32], generation: 0,
                }.into());
            for _ in 0..20_000 { value = ResourceValue::List(vec![value]); }
            assert!(matches!(check_reference_value_targets(&value, &BTreeMap::new()),
                Err(Failure::SubjectCustodyInvalid)));
            let subjects = BTreeMap::from([([3; 32], SubjectValueCandidateV1 {
                target_type: vec![1], value: ResourceValue::Bool(true),
            })]);
            assert!(check_reference_value_targets(&value, &subjects).is_ok());
            value.drop_stack_safe();
        }).unwrap().join().unwrap();
    }

    #[test]
    fn live_resource_table_is_bounded_and_uses_existing_saved_value_codec(){
        let values=BTreeMap::from([
            ("a".into(),ResourceValue::Fixed64(Fixed64::ONE)),
            ("b".into(),ResourceValue::List(vec![ResourceValue::String("추".into()),ResourceValue::Bool(true)])),
        ]);
        let encoded=encode_resource_table(&values).unwrap();
        assert_eq!(decode_resource_table(&encoded).unwrap(),values);
        let too_large=BTreeMap::from([("a".into(),ResourceValue::String("x".repeat(TOTAL_MAX)))]);
        assert!(matches!(encode_resource_table(&too_large),Err(Failure::LimitExceeded)));
        let handle=BTreeMap::from([("a".into(),ResourceValue::ResourceHandle(ddonirang_core::ResourceHandle::from_raw(1)))]);
        // Existing raw-handle codec support is preserved. A table candidate
        // is not custody admission; the live-world profile gate is separate.
        assert_eq!(decode_resource_table(&encode_resource_table(&handle).unwrap()).unwrap(),handle);
    }
    #[test]
    fn beta1_reference_targets_are_checked_in_all_saved_collection_positions(){
        use ddonirang_core::platform::{IdentityReferenceValueV1,ResourceMapEntry};
        let id=[3;32];
        let reference=ResourceValue::IdentityReference(IdentityReferenceValueV1{
            profile_version:1,target_type_identity:vec![0x11,0],
            owner_space_identity:vec![4;32],target_identity:id.to_vec(),generation:0,
        }.into());
        let subjects=BTreeMap::from([(id,SubjectValueCandidateV1{
            target_type:vec![0x11,0],value:ResourceValue::None,
        })]);
        let list=ResourceValue::List(vec![reference.clone()]);
        let set=ResourceValue::Set(BTreeMap::from([("key".into(),reference.clone())]));
        let key=ResourceValue::Map(BTreeMap::from([("key".into(),ResourceMapEntry{
            key:reference.clone(),value:ResourceValue::None,
        })]));
        let value=ResourceValue::Map(BTreeMap::from([("key".into(),ResourceMapEntry{
            key:ResourceValue::None,value:reference.clone(),
        })]));
        let record=ResourceValue::Record{field_types:BTreeMap::new(),
            fields:BTreeMap::from([("field".into(),reference.clone())])};
        // This is the joint target-presence check, not individual codec/type
        // admission; malformed Record/Set spellings are checked earlier.
        for value in [reference,list,set,key,value,record] {
            assert!(check_reference_value_targets(&value,&subjects).is_ok());
            assert!(matches!(check_reference_value_targets(&value,&BTreeMap::new()),Err(Failure::SubjectCustodyInvalid)));
            let wrong=BTreeMap::from([(id,SubjectValueCandidateV1{
                target_type:vec![0x12,0],value:ResourceValue::None,
            })]);
            assert!(matches!(check_reference_value_targets(&value,&wrong),Err(Failure::SubjectCustodyInvalid)));
        }
        assert!(check_reference_value_targets(&ResourceValue::None,&BTreeMap::new()).is_ok());
    }
    #[test]
    fn beta1_subject_resource_parameter_tables_share_one_materialization_budget(){
        let mut resource=1u64.to_be_bytes().to_vec();resource.extend(1u32.to_be_bytes());resource.push(b'a');
        resource.extend(1u64.to_be_bytes());resource.push(0);
        let parameter=encode_parameter_table(&BTreeMap::from([("b".into(),Fixed64::ONE)])).unwrap();
        let mut budget=super::super::saved_resource_value::ValueDecodeBudget::new();
        // Prior channels have already used nearly all of the same budget.
        budget.charge(16_000_000-220).unwrap();
        assert!(decode_resource_table_with_custody(&resource,&[],None,&mut budget).is_ok());
        assert!(matches!(decode_parameter_table(&parameter,&mut budget),Err(Failure::LimitExceeded)));
        // Independent channels fit; only their combined allocation is rejected.
        assert!(decode_parameter_table(&parameter,&mut super::super::saved_resource_value::ValueDecodeBudget::new()).is_ok());
    }
    #[test]
    fn beta1_subject_value_table_keeps_values_and_requires_live_recorded_custody() {
        use crate::subject_reference_owner::{SavedSubjectRow,SavedWorldIdentityCustodyV1};
        let first=[1;32];let second=[2;32];
        let mut custody=SavedWorldIdentityCustodyV1{nonce:[3;32],world:[4;32],pesi:[5;32],next_subject_ordinal:2,
            rows:vec![SavedSubjectRow{identity:second,ordinal:0,live:true,target_type:vec![0x11,0]},
                SavedSubjectRow{identity:first,ordinal:1,live:true,target_type:vec![0x11,0]}]};
        // This tests the subordinate table owner, not custody identity issuance.
        let value=ResourceValue::Fixed64(Fixed64::from_raw_i64(-7));
        let mut raw=Vec::new();value.encode_canon(&mut raw);
        let row=|id:[u8;32]|{let mut row=id.to_vec();row.extend((raw.len() as u64).to_be_bytes());row.extend(&raw);row};
        let mut table=2u64.to_be_bytes().to_vec();table.extend(row(first));table.extend(row(second));
        let values=decode_subject_value_table(&table,Some(&custody),&[]).unwrap();
        assert_eq!(values.len(),2);assert_eq!(values[&first].value,value);
        assert_eq!(values[&second].target_type,[0x11,0]);
        assert!(decode_subject_value_table(&table,None,&[]).is_err());
        custody.rows[0].live=false;assert!(decode_subject_value_table(&table,Some(&custody),&[]).is_err());
        custody.rows[0].live=true;
        for n in 0..table.len(){assert!(decode_subject_value_table(&table[..n],Some(&custody),&[]).is_err(),"truncation {n}");}
        let mut suffix=table.clone();suffix.push(0);assert!(decode_subject_value_table(&suffix,Some(&custody),&[]).is_err());
        for ids in [[first,first],[second,first],[first,[9;32]]] {
            let mut bad=2u64.to_be_bytes().to_vec();for id in ids {bad.extend(row(id));}
            assert!(decode_subject_value_table(&bad,Some(&custody),&[]).is_err());
        }
        let mut wrong=table.clone();wrong[48]=0xff;
        assert!(decode_subject_value_table(&wrong,Some(&custody),&[]).is_err());
        let mut oversized=table.clone();oversized[40..48].copy_from_slice(&2_000_001u64.to_be_bytes());
        assert!(matches!(decode_subject_value_table(&oversized,Some(&custody),&[]),Err(Failure::LimitExceeded)));
        assert!(matches!(decode_subject_value_table(&65_537u64.to_be_bytes(),None,&[]),Err(Failure::LimitExceeded)));
        assert!(decode_subject_value_table(&[0;8],Some(&custody),&[]).unwrap().is_empty());
        assert_eq!(custody.rows.len(),2,"empty table never erases allocator/tombstones");
    }
    fn atom_evidence(raw:&[u8],custody:&crate::registered_atom_contract::ContractCustody)->Vec<u8>{
        let mut bytes=1u64.to_be_bytes().to_vec();bytes.extend(custody.snapshot_identity);
        bytes.extend(custody.sha256);bytes.extend(custody.byte_length.to_be_bytes());bytes.extend(raw);bytes
    }
    #[test]
    fn beta1_atom_section_a_uses_exact_artifact_and_rejects_order_suffix_and_limits(){
        let (raw,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"a",&[vec![1]]).unwrap();
        assert_eq!(raw.len(),160);
        assert_eq!(hex::encode(contract.custody().sha256),"712380168dce69f9cc853aa01666842862cb353f6bb72b54319942d88f586b1c");
        let bytes=atom_evidence(&raw,contract.custody());
        assert_eq!(contract.recorded_bytes(), raw);
        assert_eq!(encode_registered_atom_evidence(&[contract.clone()]).unwrap(), bytes);
        assert_eq!(encode_registered_atom_evidence(&[]).unwrap(), 0u64.to_be_bytes());
        let contracts=decode_registered_atom_evidence(&bytes).unwrap();
        assert_eq!(contracts.len(),1);assert_eq!(contracts[0].custody(),contract.custody());
        assert!(decode_registered_atom_evidence(&[]).unwrap().is_empty());
        assert!(decode_registered_atom_evidence(&0u64.to_be_bytes()).unwrap().is_empty());
        for n in 1..bytes.len(){assert!(decode_registered_atom_evidence(&bytes[..n]).is_err(),"truncation {n}");}
        let mut suffix=bytes.clone();suffix.push(0);assert!(decode_registered_atom_evidence(&suffix).is_err());
        for offset in [8,40,80]{let mut bad=bytes.clone();bad[offset]^=1;assert!(decode_registered_atom_evidence(&bad).is_err(),"mutated {offset}");}
        let mut oversized=bytes.clone();oversized[72..80].copy_from_slice(&1_000_001u64.to_be_bytes());
        assert!(matches!(decode_registered_atom_evidence(&oversized),Err(Failure::LimitExceeded)));
        assert!(matches!(decode_registered_atom_evidence(&257u64.to_be_bytes()),Err(Failure::LimitExceeded)));
        let mut duplicate=2u64.to_be_bytes().to_vec();duplicate.extend(&bytes[8..]);duplicate.extend(&bytes[8..]);
        assert!(decode_registered_atom_evidence(&duplicate).is_err());
        let (next_raw,next)=crate::registered_atom_contract::RecordedAtomContract::record(b"a",&[vec![1],vec![2]]).unwrap();
        let second=atom_evidence(&next_raw,next.custody());
        for (first,second) in [(&bytes,&second),(&second,&bytes)]{
            let mut conflict=2u64.to_be_bytes().to_vec();conflict.extend(&first[8..]);conflict.extend(&second[8..]);
            assert!(decode_registered_atom_evidence(&conflict).is_err());
        }
    }
    #[test]
    fn beta1_atom_table_requires_exact_contract_before_materialization() {
        let (_,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"owner",&[vec![1]]).unwrap();
        let atom=ResourceValue::RegisteredAtom(contract.admitted_value(b"owner",&[1]).unwrap().into());
        let mut raw=Vec::new();atom.encode_canon(&mut raw);
        let mut table=1u64.to_be_bytes().to_vec();table.extend_from_slice(&3u32.to_be_bytes());table.extend_from_slice("값".as_bytes());
        table.extend_from_slice(&(raw.len() as u64).to_be_bytes());table.extend_from_slice(&raw);
        assert!(decode_resource_table(&table).is_err());
        assert_eq!(decode_resource_table_with_contracts(&table,&[contract.clone()]).unwrap()["값"],atom);
        let (_,successor)=crate::registered_atom_contract::RecordedAtomContract::record(b"owner",&[vec![1],vec![2]]).unwrap();
        assert!(decode_resource_table_with_contracts(&table,&[successor]).is_err());
        *table.last_mut().unwrap()^=1;
        assert!(decode_resource_table_with_contracts(&table,&[contract]).is_err());
    }
    #[test]
    fn rd670_resource_table_uses_actual_core_value_bytes_and_rejects_invalid_leaf() {
        let value=ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::from_raw_i64(-1)),ResourceValue::Bool(true)]);
        let mut raw=Vec::new();value.encode_canon(&mut raw);
        let mut table=1u64.to_be_bytes().to_vec();table.extend_from_slice(&3u32.to_be_bytes());table.extend_from_slice("값".as_bytes());
        table.extend_from_slice(&(raw.len() as u64).to_be_bytes());table.extend_from_slice(&raw);
        assert_eq!(decode_resource_table(&table).unwrap()["값"],value);
        *table.last_mut().unwrap()=2;
        assert!(check_resource_table_rows(&table).is_ok());
        assert!(decode_resource_table(&table).is_err());
    }
    #[test]
    fn rd670_parameters_exact_raw_roundtrip_and_independent_be_fixture() {
        let values = BTreeMap::from([("a".into(),Fixed64::from_raw_i64(i64::MIN)),
            ("속도".into(),Fixed64::from_raw_i64(i64::MAX))]);
        let bytes = encode_parameter_table(&values).unwrap();
        let mut expected = 2u64.to_be_bytes().to_vec();
        expected.extend_from_slice(&1u32.to_be_bytes()); expected.push(b'a');
        expected.extend_from_slice(&i64::MIN.to_be_bytes());
        expected.extend_from_slice(&6u32.to_be_bytes()); expected.extend_from_slice("속도".as_bytes());
        expected.extend_from_slice(&i64::MAX.to_be_bytes());
        assert_eq!(bytes,expected);
        assert_eq!(decode_parameter_table(&bytes,&mut super::super::saved_resource_value::ValueDecodeBudget::new()).unwrap(),values);
        for n in 0..bytes.len() { assert!(decode_parameter_table(&bytes[..n],&mut super::super::saved_resource_value::ValueDecodeBudget::new()).is_err()); }
        let mut bad=bytes.clone();bad.push(0);assert!(decode_parameter_table(&bad,&mut super::super::saved_resource_value::ValueDecodeBudget::new()).is_err());
    }
    #[test]
    fn rd670_tables_reject_duplicates_wrong_order_utf8_and_empty_values() {
        fn rows(names:&[&[u8]], resource:bool)->Vec<u8> {
            let mut bytes=(names.len() as u64).to_be_bytes().to_vec();
            for name in names {
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes()); bytes.extend_from_slice(name);
                if resource {bytes.extend_from_slice(&1u64.to_be_bytes());bytes.push(0);}
                else {bytes.extend_from_slice(&0i64.to_be_bytes());}
            }
            bytes
        }
        for resource in [false,true] {
            let check=if resource {check_resource_table_rows} else {check_parameter_table};
            assert!(check(&rows(&[b"a",b"b"],resource)).is_ok());
            for names in [vec![&b"a"[..],&b"a"[..]],vec![&b"b"[..],&b"a"[..]],vec![&[255][..]]] {
                assert!(check(&rows(&names,resource)).is_err());
            }
            let mut suffix=rows(&[b"a"],resource);suffix.push(0);assert!(check(&suffix).is_err());
        }
        let mut empty=1u64.to_be_bytes().to_vec();empty.extend_from_slice(&1u32.to_be_bytes());empty.push(b'a');
        empty.extend_from_slice(&0u64.to_be_bytes());assert!(check_resource_table_rows(&empty).is_err());
    }
}
