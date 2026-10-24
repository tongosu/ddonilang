//! R5 genuine canonical value carriers. These are data, not catalog admission
//! or identity issuers. The shared runtime must bind every ID/type to its
//! recorded catalog before construction or restoration becomes usable.
use crate::platform::{RecordFieldType, ResourceValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NominalSlotTableV1 {
    name_type_identity: [u8; 32],
    body: Vec<u8>,
    node_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NominalCarrierError { LimitExceeded, DuplicateOrUnsortedSlot, SubjectPayloadInValue }

impl NominalSlotTableV1 {
    /// Takes resolved slots in exact SlotID order, not declaration spelling.
    /// No arbitrary-byte constructor exists: children use the common writer.
    pub fn from_slots(
        name_type_identity: [u8; 32],
        slots: &[([u8; 32], RecordFieldType, ResourceValue)],
        byte_limit: usize,
        node_budget: usize,
    ) -> Result<Self, NominalCarrierError> {
        use NominalCarrierError as Error;
        if byte_limit < 42 || node_budget == 0 {
            return Err(Error::LimitExceeded);
        }
        if slots.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err(Error::DuplicateOrUnsortedSlot);
        }
        let mut remaining_nodes = node_budget - 1;
        // Preflight all children before allocating the output; a bounded probe
        // uses the same canonical writer and a shared materialization budget.
        let mut length = 42usize; // version + NameTypeID + slot count
        for (_, kind, child) in slots {
            let (type_length, type_nodes) = kind.canonical_measure(byte_limit, remaining_nodes).ok_or(Error::LimitExceeded)?;
            remaining_nodes = remaining_nodes.checked_sub(type_nodes).ok_or(Error::LimitExceeded)?;
            let (child_length, child_nodes) = child.canonical_measure(byte_limit, remaining_nodes).ok_or(Error::LimitExceeded)?;
            remaining_nodes = remaining_nodes.checked_sub(child_nodes).ok_or(Error::LimitExceeded)?;
            let mut pending = vec![child];
            while let Some(value) = pending.pop() {
                match value {
                    ResourceValue::SubjectPayloadV1(_) => return Err(Error::SubjectPayloadInValue),
                    ResourceValue::List(items) | ResourceValue::Tuple { items, .. } => pending.extend(items),
                    ResourceValue::Set(items) => pending.extend(items.values()),
                    ResourceValue::Map(items) => for entry in items.values() { pending.push(&entry.key); pending.push(&entry.value); },
                    ResourceValue::Record { fields, .. } => pending.extend(fields.values()),
                    _ => {},
                }
            }
            length = length.checked_add(48).and_then(|n| n.checked_add(type_length))
                .and_then(|n| n.checked_add(child_length)).filter(|n| *n <= byte_limit).ok_or(Error::LimitExceeded)?;
        }
        let mut body = Vec::with_capacity(length);
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&name_type_identity);
        body.extend_from_slice(&(slots.len() as u64).to_le_bytes());
        for (slot, kind, child) in slots {
            body.extend_from_slice(slot);
            let type_length = kind.canonical_measure(byte_limit, node_budget).ok_or(Error::LimitExceeded)?.0;
            body.extend_from_slice(&(type_length as u64).to_le_bytes());
            kind.encode_canon(&mut body);
            let child_length = child.canonical_byte_length(byte_limit, node_budget).ok_or(Error::LimitExceeded)?;
            body.extend_from_slice(&(child_length as u64).to_le_bytes());
            child.encode_canon(&mut body);
        }
        debug_assert_eq!(body.len(), length);
        Ok(Self { name_type_identity, body, node_count:node_budget-remaining_nodes })
    }

    pub fn name_type_identity(&self) -> &[u8; 32] { &self.name_type_identity }
    pub fn canonical_body(&self) -> &[u8] { &self.body }
    pub fn node_count(&self) -> usize { self.node_count }
}

/// A Subject value is an identity-bearing handle, never a payload or an
/// explicit 가리킴. Generation is fixed to zero by this R5 profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectHandleV1 {
    pub name_type_identity: [u8; 32],
    pub world_identity: [u8; 32],
    pub subject_identity: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn boolean_slot(id:u8)->([u8;32],RecordFieldType,ResourceValue) {
        ([id;32],RecordFieldType::Named{name:"참거짓".into()},ResourceValue::Bool(true))
    }
    #[test]
    fn r5_exact_nominal_payload_and_handle_are_distinct() {
        let slots=[boolean_slot(2)];
        let table=Arc::new(NominalSlotTableV1::from_slots([1;32],&slots,4096,20).unwrap());
        let nominal=ResourceValue::NominalValueV1(table.clone());
        let payload=ResourceValue::SubjectPayloadV1(table.clone());
        let mut expected=vec![0x11,1,0];expected.extend_from_slice(&[1;32]);
        expected.extend_from_slice(&1u64.to_le_bytes());expected.extend_from_slice(&[2;32]);
        let mut ty=Vec::new();slots[0].1.encode_canon(&mut ty);
        expected.extend_from_slice(&(ty.len() as u64).to_le_bytes());expected.extend_from_slice(&ty);
        expected.extend_from_slice(&2u64.to_le_bytes());expected.extend_from_slice(&[1,1]);
        let mut actual=Vec::new();nominal.encode_canon(&mut actual);assert_eq!(actual,expected);
        assert_eq!(nominal.canonical_byte_length(actual.len(),3),Some(actual.len()));
        assert_eq!(nominal.canonical_byte_length(actual.len(),2),None);
        let mut payload_bytes=Vec::new();payload.encode_canon(&mut payload_bytes);
        assert_eq!(payload_bytes[0],0x13);assert_eq!(&payload_bytes[1..],&actual[1..]);
        assert_ne!(nominal,payload);assert_ne!(nominal.canon_key(),payload.canon_key());
        let other=ResourceValue::NominalValueV1(Arc::new(NominalSlotTableV1::from_slots([3;32],&slots,4096,20).unwrap()));
        assert_ne!(nominal,other);assert_ne!(nominal.canon_key(),other.canon_key());
        let handle=ResourceValue::SubjectHandleV1(Box::new(SubjectHandleV1{name_type_identity:[1;32],world_identity:[2;32],subject_identity:[3;32]}));
        let mut bytes=Vec::new();handle.encode_canon(&mut bytes);
        assert_eq!(bytes.len(),107);assert_eq!(&bytes[..3],&[0x12,1,0]);
        assert_eq!(&bytes[3..35],&[1;32]);assert_eq!(&bytes[35..67],&[2;32]);
        assert_eq!(&bytes[67..99],&[3;32]);assert_eq!(&bytes[99..],&0u64.to_le_bytes());
    }
    #[test]
    fn r5_slots_reject_duplicate_unsorted_payload_and_excess_budget() {
        use NominalCarrierError as E;
        for slots in [vec![boolean_slot(2),boolean_slot(2)],vec![boolean_slot(3),boolean_slot(2)]] {
            assert_eq!(NominalSlotTableV1::from_slots([1;32],&slots,4096,20),Err(E::DuplicateOrUnsortedSlot));
        }
        let slots=[boolean_slot(2),boolean_slot(3)];
        assert_eq!(NominalSlotTableV1::from_slots([1;32],&slots,4096,4),Err(E::LimitExceeded));
        let table=NominalSlotTableV1::from_slots([1;32],&slots,4096,5).unwrap();
        assert_eq!(table.node_count(),5);
        assert_eq!(NominalSlotTableV1::from_slots([1;32],&slots,table.canonical_body().len()-1,5),Err(E::LimitExceeded));
        let bad=[([2;32],RecordFieldType::Named{name:"글".into()},ResourceValue::List(vec![ResourceValue::SubjectPayloadV1(Arc::new(table))]))];
        assert_eq!(NominalSlotTableV1::from_slots([1;32],&bad,8192,100),Err(E::SubjectPayloadInValue));
    }
    #[test]
    fn r5_nominal_clone_drop_and_nested_preflight_are_stack_safe() {
        std::thread::Builder::new().stack_size(128*1024).spawn(|| {
            let table=Arc::new(NominalSlotTableV1::from_slots([1;32],&[boolean_slot(2)],4096,10).unwrap());
            let mut kind=RecordFieldType::ResolvedNominalV1{category:2,name_type_identity:[1;32]};
            for _ in 0..20_000 {kind=RecordFieldType::Applied{name:"차림".into(),args:vec![kind]};}
            let measured=kind.canonical_measure(1_000_000,20_001).unwrap();assert_eq!(measured.1,20_001);
            assert!(kind.canonical_measure(1_000_000,20_000).is_none());
            let mut value=ResourceValue::NominalValueV1(table);
            for _ in 0..20_000 {value=ResourceValue::List(vec![value]);}
            let copy=value.clone();assert_eq!(value,copy);
            assert!(value.canonical_measure(1_000_000,20_002).is_none());
            assert_eq!(value.canonical_measure(1_000_000,20_003).unwrap().1,20_003);
            drop(copy);drop(value);drop(kind);
        }).unwrap().join().unwrap();
    }
}
