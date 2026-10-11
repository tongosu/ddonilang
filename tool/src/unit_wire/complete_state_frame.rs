//! RD-670 / R1.3 exact complete-state framing, subordinate to unit_wire.
//! A borrowed structural view is not typed value/custody or restore admission.
//! No JSON, checksum wrapper, adapter verdict or invented identity is accepted.

use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use super::complete_state_tables::{check_resource_table_rows, check_parameter_table, decode_resource_table, decode_parameter_table};
use ddonirang_core::{Fixed64,platform::ResourceValue};
use std::collections::BTreeMap;

const DOMAIN: &[u8] = b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V1\0";
const TOTAL_MAX: usize = 2_000_000;
const SUBJECT_MAX: u64 = 65_536;
const CUSTODY_MAX: usize = 1_000_000;

/// Raw fields retained by borrow. Downstream shared typed codecs must validate
/// resource grammar, evidence, Subject custody and their cross-bindings.
#[derive(Debug)]
pub(crate) struct CompleteStateFrameV1<'a> {
    pub(crate) pesi: &'a [u8; 32],
    pub(crate) resources: &'a [u8],
    pub(crate) parameters: &'a [u8],
    pub(crate) next_tick: u64,
    pub(crate) subjects: &'a [u8],
    pub(crate) atom_evidence: &'a [u8],
    pub(crate) subject_custody: &'a [u8],
}

/// Value candidate only. Subject/atom/handle custody and revision admission are
/// still required before transaction prepare or any live-world publication.
#[derive(Debug)]
pub(crate) struct CompleteStateValueTablesV1 {
    pub(crate) resources:BTreeMap<String,ResourceValue>,
    pub(crate) parameters:BTreeMap<String,Fixed64>,
}
impl CompleteStateFrameV1<'_> {
    pub(crate) fn decode_value_tables(&self)->Result<CompleteStateValueTablesV1,Failure> {
        let resources=decode_resource_table(self.resources)?;
        let parameters=decode_parameter_table(self.parameters)?;
        Ok(CompleteStateValueTablesV1{resources,parameters})
    }
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], Failure> {
        let end = self.offset.checked_add(length).ok_or(Failure::LimitExceeded)?;
        let value = self.bytes.get(self.offset..end).ok_or(Failure::InvalidInput)?;
        self.offset = end;
        Ok(value)
    }
    fn u64(&mut self) -> Result<u64, Failure> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn field(&mut self, tag: u8) -> Result<&'a [u8], Failure> {
        if self.take(1)? != [tag] { return Err(Failure::InvalidInput); }
        let length = usize::try_from(self.u64()?).map_err(|_|Failure::LimitExceeded)?;
        if length > TOTAL_MAX { return Err(Failure::LimitExceeded); }
        self.take(length)
    }
    fn finish(&self) -> Result<(), Failure> {
        if self.offset == self.bytes.len() { Ok(()) } else { Err(Failure::InvalidInput) }
    }
}

fn table_count(raw: &[u8]) -> Result<u64, Failure> {
    let head = raw.get(..8).ok_or(Failure::InvalidInput)?;
    Ok(u64::from_be_bytes(head.try_into().map_err(|_|Failure::InvalidInput)?))
}

/// Framing, host-size conversions and common allocation bounds precede all
/// row materialization. This function allocates nothing, even on malformed
/// lengths. PESI comes from the actual issuer and is compared byte-for-byte.
pub(crate) fn check_complete_state_frame_v1<'a>(
    bytes: &'a [u8], admitted_pesi: &[u8;32],
) -> Result<CompleteStateFrameV1<'a>, Failure> {
    if bytes.len() > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(DOMAIN.len())? != DOMAIN || cursor.take(2)? != [0,1]
        || cursor.take(1)? != [8] { return Err(Failure::InvalidInput); }
    let fields = [cursor.field(1)?,cursor.field(2)?,cursor.field(3)?,cursor.field(4)?,
        cursor.field(5)?,cursor.field(6)?,cursor.field(7)?,cursor.field(8)?];
    cursor.finish()?;
    if fields[0].len()!=32 || fields[3].len()!=8 || fields[7]!=[0] {
        return Err(Failure::InvalidInput);
    }
    // Counts cannot exceed the bytes available for at least each row's length
    // prefixes; semantic table parsing subsequently checks the exact row body.
    for table in [fields[1], fields[2]] {
        let count = table_count(table)?;
        if count > (table.len().saturating_sub(8)/12) as u64 {
            return Err(Failure::InvalidInput);
        }
    }
    let subjects = table_count(fields[4])?;
    if subjects > SUBJECT_MAX { return Err(Failure::LimitExceeded); }
    if subjects > (fields[4].len().saturating_sub(8)/40) as u64 {
        return Err(Failure::InvalidInput);
    }
    if !fields[5].is_empty() && table_count(fields[5])? > 256 {
        return Err(Failure::LimitExceeded);
    }
    if fields[6].len() > CUSTODY_MAX { return Err(Failure::LimitExceeded); }
    let pesi: &[u8;32] = fields[0].try_into().map_err(|_|Failure::InvalidInput)?;
    if pesi != admitted_pesi { return Err(Failure::ForeignPesi); }
    check_resource_table_rows(fields[1])?;
    check_parameter_table(fields[2])?;
    Ok(CompleteStateFrameV1 { pesi, resources:fields[1], parameters:fields[2],
        next_tick:u64::from_be_bytes(fields[3].try_into().map_err(|_|Failure::InvalidInput)?),
        subjects:fields[4], atom_evidence:fields[5], subject_custody:fields[6] })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let mut bytes=DOMAIN.to_vec(); bytes.extend_from_slice(&[0,1,8]);
        let empty=[0u8;8]; let pesi=[7u8;32]; let tick=3u64.to_be_bytes();
        for (index,field) in [&pesi[..],&empty,&empty,&tick,&empty,&[][..],&[][..],&[0][..]].iter().enumerate() {
            bytes.push(index as u8+1); bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    }
    #[test]
    fn rd670_complete_frame_borrows_exact_eight_fields() {
        let bytes=fixture(); let frame=check_complete_state_frame_v1(&bytes,&[7;32]).unwrap();
        assert_eq!(frame.next_tick,3); assert_eq!(frame.pesi,&[7;32]);
        assert_eq!(frame.resources,[0;8]); assert_eq!(frame.parameters,[0;8]);
        assert_eq!(frame.subjects,[0;8]); assert!(frame.atom_evidence.is_empty());
        assert!(frame.subject_custody.is_empty());
        assert!(matches!(check_complete_state_frame_v1(&bytes,&[8;32]),Err(Failure::ForeignPesi)));
    }
    #[test]
    fn rd670_complete_frame_rejects_wrappers_lengths_order_and_suffix() {
        let bytes=fixture();
        for length in 0..bytes.len() {
            assert!(check_complete_state_frame_v1(&bytes[..length],&[7;32]).is_err());
        }
        for offset in [0,DOMAIN.len(),DOMAIN.len()+1,DOMAIN.len()+2,DOMAIN.len()+3] {
            let mut bad=bytes.clone(); bad[offset]^=1;
            assert!(check_complete_state_frame_v1(&bad,&[7;32]).is_err());
        }
        let mut bad=bytes.clone(); bad.extend_from_slice(&[0]);
        assert!(check_complete_state_frame_v1(&bad,&[7;32]).is_err());
        let mut bad=bytes.clone(); bad[DOMAIN.len()+4..DOMAIN.len()+12].copy_from_slice(&u64::MAX.to_be_bytes());
        assert!(matches!(check_complete_state_frame_v1(&bad,&[7;32]),Err(Failure::LimitExceeded)));
        let mut bad=bytes; *bad.last_mut().unwrap()=1;
        assert!(check_complete_state_frame_v1(&bad,&[7;32]).is_err());
    }
}
