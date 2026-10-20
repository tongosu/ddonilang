//! RD-670 / R1.3 exact complete-state framing, subordinate to unit_wire.
//! A borrowed structural view is not typed value/custody or restore admission.
//! No JSON, checksum wrapper, adapter verdict or invented identity is accepted.

use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use super::complete_state_tables::{check_resource_table_rows, check_parameter_table, decode_resource_table_with_contracts, decode_registered_atom_evidence, decode_parameter_table,decode_saved_world_custody};
use ddonirang_core::{Fixed64,platform::ResourceValue};
use std::collections::BTreeMap;

const DOMAIN: &[u8] = b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V1\0";
const RESUMABLE_DOMAIN: &[u8] = b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V2\0";
const RESUMABLE_INTEGRITY_DOMAIN: &[u8] = b"DDN-RESUMABLE-COMPLETE-STATE-INTEGRITY-V2\0";
const TOTAL_MAX: usize = 2_000_000;
const SUBJECT_MAX: u64 = 65_536;
const CUSTODY_MAX: usize = 1_000_000;

/// Raw fields retained by borrow. Downstream shared typed codecs must validate
/// resource grammar, evidence, Subject custody and their cross-bindings.
#[derive(Debug,Clone,Copy)]
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
    pub(crate) subject_custody:Option<crate::subject_reference_owner::SavedWorldIdentityCustodyV1>,
    pub(crate) subjects:BTreeMap<[u8;32],super::complete_state_tables::SubjectValueCandidateV1>,
}
impl CompleteStateFrameV1<'_> {
    /// Existing eight owner payloads for SEED1; this grants no restore right.
    pub(crate) fn replay_seed_world_payloads(&self)->Result<[Vec<u8>;8],Failure> {
        self.encode_canonical()?;
        Ok([self.pesi.to_vec(),self.resources.to_vec(),self.parameters.to_vec(),
            self.next_tick.to_be_bytes().to_vec(),self.subjects.to_vec(),
            self.atom_evidence.to_vec(),self.subject_custody.to_vec(),vec![0]])
    }
    pub(crate) fn decode_value_tables(&self)->Result<CompleteStateValueTablesV1,Failure> {
        self.decode_value_tables_with_catalog(None)
    }
    pub(crate) fn decode_value_tables_with_catalog(&self,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<CompleteStateValueTablesV1,Failure> {
        check_resource_table_rows(self.resources)?;
        check_parameter_table(self.parameters)?;
        super::complete_state_tables::check_subject_value_table_rows(self.subjects)?;
        let contracts=decode_registered_atom_evidence(self.atom_evidence)?;
        let subject_custody=decode_saved_world_custody(self.subject_custody,self.pesi)?;
        let references=subject_custody.as_ref().map(crate::subject_reference_owner::SubjectReferenceContextV1::new).transpose()?;
        let mut budget=super::saved_resource_value::ValueDecodeBudget::new();
        let resources=super::complete_state_tables::decode_resource_table_with_catalog(self.resources,&contracts,references.as_ref(),&mut budget,catalog)?;
        let parameters=decode_parameter_table(self.parameters,&mut budget)?;
        let subjects=super::complete_state_tables::decode_subject_value_table_with_catalog(self.subjects,subject_custody.as_ref(),&contracts,references.as_ref(),&mut budget,catalog)?;
        budget.check_targets(&subjects)?;
        // Joint validation must wait until every row exists, so forward/self
        // references do not depend on serialization order.
        for value in resources.values().chain(subjects.values().map(|row|&row.value)) {
            super::complete_state_tables::check_reference_value_targets(value,&subjects)?;
        }
        Ok(CompleteStateValueTablesV1{resources,parameters,subject_custody,subjects})
    }

    /// Common Rust producer for the already specified eight-field frame.
    /// This does not issue PESI/custody, prepare a transaction or authorize a
    /// restore. Both framing and typed tables are checked before returning.
    pub(crate) fn encode_canonical(&self) -> Result<Vec<u8>,Failure> {
        self.encode_canonical_with_catalog(None)
    }
    pub(crate) fn encode_canonical_with_catalog(&self,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>) -> Result<Vec<u8>,Failure> {
        let tick=self.next_tick.to_be_bytes();
        let fields:[&[u8];8]=[self.pesi,self.resources,self.parameters,&tick,
            self.subjects,self.atom_evidence,self.subject_custody,&[0]];
        let mut length=DOMAIN.len()+3;
        for field in fields {
            length=length.checked_add(9).and_then(|n|n.checked_add(field.len()))
                .ok_or(Failure::LimitExceeded)?;
            if length>TOTAL_MAX {return Err(Failure::LimitExceeded);}
        }
        // Reject malformed/oversized values before copying their byte bodies.
        self.decode_value_tables_with_catalog(catalog)?;
        check_subject_and_custody_bounds(self.subjects,self.atom_evidence,self.subject_custody)?;
        let mut bytes=Vec::new();
        bytes.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend_from_slice(DOMAIN);
        bytes.extend_from_slice(&[0,1,8]);
        for (index,field) in fields.into_iter().enumerate() {
            bytes.push(index as u8+1);
            bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());
            bytes.extend_from_slice(field);
        }
        check_complete_state_frame_v1(&bytes,self.pesi)?;
        Ok(bytes)
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

fn check_subject_and_custody_bounds(subjects:&[u8],atom_evidence:&[u8],subject_custody:&[u8]) -> Result<(),Failure> {
    let count=table_count(subjects)?;
    if count>SUBJECT_MAX {return Err(Failure::LimitExceeded);}
    if count>(subjects.len().saturating_sub(8)/40) as u64 {return Err(Failure::InvalidInput);}
    if !atom_evidence.is_empty() && table_count(atom_evidence)?>256 {return Err(Failure::LimitExceeded);}
    if subject_custody.len()>CUSTODY_MAX {return Err(Failure::LimitExceeded);}
    Ok(())
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
    check_world_fields(fields, admitted_pesi)
}

// V1 and the successor share the existing eight payload grammars, not an
// enclosing container or a migration. Both paths use this exact validator.
fn check_world_fields<'a>(fields: [&'a [u8]; 8], admitted_pesi: &[u8;32])
    -> Result<CompleteStateFrameV1<'a>, Failure> {
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
    check_subject_and_custody_bounds(fields[4],fields[5],fields[6])?;
    super::complete_state_tables::check_subject_value_table_rows(fields[4])?;
    let pesi: &[u8;32] = fields[0].try_into().map_err(|_|Failure::InvalidInput)?;
    if pesi != admitted_pesi { return Err(Failure::ForeignPesi); }
    check_resource_table_rows(fields[1])?;
    check_parameter_table(fields[2])?;
    Ok(CompleteStateFrameV1 { pesi, resources:fields[1], parameters:fields[2],
        next_tick:u64::from_be_bytes(fields[3].try_into().map_err(|_|Failure::InvalidInput)?),
        subjects:fields[4], atom_evidence:fields[5], subject_custody:fields[6] })
}

/// Borrowed layout only: not source, continuation, checkpoint provenance or
/// durability admission. No adapter may treat this result as a resume token.
#[derive(Debug)]
pub(crate) struct ResumableCompleteStateLayoutV2<'a> {
    pub(crate) world: CompleteStateFrameV1<'a>,
    pub(crate) supplies: [&'a [u8]; 10], // fields 09..12, exact owner bytes
}

pub(crate) fn check_resumable_complete_state_layout_v2<'a>(
    bytes: &'a [u8], admitted_pesi: &[u8;32],
) -> Result<ResumableCompleteStateLayoutV2<'a>, Failure> {
    use sha2::{Digest, Sha256};
    if bytes.len() > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(RESUMABLE_DOMAIN.len())? != RESUMABLE_DOMAIN
        || cursor.take(2)? != [0,2] || cursor.take(1)? != [19] {
        return Err(Failure::InvalidInput);
    }
    let mut fields = [&[][..]; 19];
    for (index, field) in fields.iter_mut().enumerate() {
        *field = cursor.field(index as u8 + 1)?;
    }
    cursor.finish()?;
    if fields[18].len() != 32 { return Err(Failure::InvalidInput); }
    // The empty-field13 frame retains tag13 and its eight zero length bytes.
    // Hash by borrowing segments: no duplicate two-megabyte allocation.
    let empty_length = bytes.len().checked_sub(32).ok_or(Failure::InvalidInput)?;
    let length_offset = empty_length.checked_sub(8).ok_or(Failure::InvalidInput)?;
    let mut digest = Sha256::new();
    digest.update(RESUMABLE_INTEGRITY_DOMAIN);
    digest.update(2u16.to_be_bytes());
    digest.update((empty_length as u64).to_be_bytes());
    digest.update(&bytes[..length_offset]);
    digest.update([0;8]);
    if digest.finalize().as_slice() != fields[18] { return Err(Failure::SnapshotHashMismatch); }
    let world = check_world_fields(fields[..8].try_into().map_err(|_|Failure::InvalidInput)?, admitted_pesi)?;
    if fields[8].len() > 1_000_000 { return Err(Failure::LimitExceeded); }
    // Named mandatory sections cannot be satisfied by an absent payload.
    // Their exact owner-native grammar/cross-checks remain the next gate.
    if fields[8..18].iter().any(|field|field.is_empty()) { return Err(Failure::InvalidInput); }
    Ok(ResumableCompleteStateLayoutV2 { world,
        supplies: fields[8..18].try_into().map_err(|_|Failure::InvalidInput)? })
}

impl ResumableCompleteStateLayoutV2<'_> {
    /// The common producer preserves the eight world payloads and adds only
    /// the approved ten supplies and integrity field. This grants no admission.
    pub(crate) fn encode_canonical(&self)->Result<Vec<u8>,Failure> {
        self.encode_canonical_with_catalog(None)
    }
    pub(crate) fn encode_canonical_with_catalog(&self,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<Vec<u8>,Failure> {
        use sha2::{Digest,Sha256};
        self.world.encode_canonical_with_catalog(catalog)?;
        let tick=self.world.next_tick.to_be_bytes();
        let world=[&self.world.pesi[..],self.world.resources,self.world.parameters,&tick[..],
            self.world.subjects,self.world.atom_evidence,self.world.subject_custody,&[0][..]];
        let fields=world.into_iter().chain(self.supplies).chain([&[][..]]);
        let mut length=RESUMABLE_DOMAIN.len()+3;
        for field in fields.clone() {
            length=length.checked_add(9).and_then(|n|n.checked_add(field.len())).ok_or(Failure::LimitExceeded)?;
        }
        if length.checked_add(32).ok_or(Failure::LimitExceeded)?>TOTAL_MAX {return Err(Failure::LimitExceeded);}
        let mut out=Vec::new();out.try_reserve_exact(length+32).map_err(|_|Failure::LimitExceeded)?;
        out.extend(RESUMABLE_DOMAIN);out.extend([0,2,19]);
        for (i,field) in fields.enumerate() {
            out.push(i as u8+1);out.extend((field.len() as u64).to_be_bytes());out.extend(field);
        }
        let mut digest=Sha256::new();digest.update(RESUMABLE_INTEGRITY_DOMAIN);
        digest.update(2u16.to_be_bytes());digest.update((out.len() as u64).to_be_bytes());digest.update(&out);
        let integrity=digest.finalize();let offset=out.len()-8;
        out[offset..].copy_from_slice(&32u64.to_be_bytes());out.extend(integrity);
        check_resumable_complete_state_layout_v2(&out,self.world.pesi)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resumable_layout_preserves_v1_payloads_and_rejects_mutated_framing() {
        use sha2::{Digest,Sha256};
        fn raw(fields: &[Vec<u8>;19], count:u8) -> Vec<u8> {
            let mut bytes=RESUMABLE_DOMAIN.to_vec();
            bytes.extend(2u16.to_be_bytes());bytes.push(count);
            for (index,field) in fields.iter().enumerate() {
                bytes.push(index as u8+1);bytes.extend((field.len() as u64).to_be_bytes());bytes.extend(field);
            }
            bytes
        }
        fn seal(fields: &mut [Vec<u8>;19]) -> Vec<u8> {
            fields[18].clear();
            let empty=raw(fields,19);
            let mut hash=Sha256::new();hash.update(RESUMABLE_INTEGRITY_DOMAIN);
            hash.update(2u16.to_be_bytes());hash.update((empty.len() as u64).to_be_bytes());hash.update(empty);
            fields[18]=hash.finalize().to_vec();raw(fields,19)
        }
        // Continuation payloads here are deliberately layout fixtures, NOT an
        // admitted capsule. No full-positive/source/runner claim is made.
        let mut fields: [Vec<u8>;19]=std::array::from_fn(|_|vec![1]);
        fields[0]=vec![7;32];fields[1]=vec![0;8];fields[2]=vec![0;8];
        fields[3]=3u64.to_be_bytes().to_vec();fields[4]=vec![0;8];
        fields[5].clear();fields[6].clear();fields[7]=vec![0];
        let bytes=seal(&mut fields);
        let layout=check_resumable_complete_state_layout_v2(&bytes,&[7;32]).unwrap();
        assert_eq!(layout.world.next_tick,3);assert_eq!(layout.supplies.len(),10);
        let old=layout.world.encode_canonical().unwrap();
        assert!(check_complete_state_frame_v1(&old,&[7;32]).is_ok());
        assert!(check_complete_state_frame_v1(&bytes,&[7;32]).is_err());
        assert!(check_resumable_complete_state_layout_v2(&old,&[7;32]).is_err());
        assert_eq!(layout.world.resources,fields[1]);assert_eq!(layout.world.parameters,fields[2]);
        let mut changed=bytes.clone();*changed.last_mut().unwrap()^=1;
        assert!(matches!(check_resumable_complete_state_layout_v2(&changed,&[7;32]),Err(Failure::SnapshotHashMismatch)));
        let mut suffix=bytes.clone();suffix.push(0);
        assert!(matches!(check_resumable_complete_state_layout_v2(&suffix,&[7;32]),Err(Failure::InvalidInput)));
        let mut wrong_tag=bytes.clone();wrong_tag[RESUMABLE_DOMAIN.len()+3]=2;
        assert!(matches!(check_resumable_complete_state_layout_v2(&wrong_tag,&[7;32]),Err(Failure::InvalidInput)));
        assert!(matches!(check_resumable_complete_state_layout_v2(&raw(&fields,18),&[7;32]),Err(Failure::InvalidInput)));
        for end in [0,RESUMABLE_DOMAIN.len(),bytes.len()-1] {
            assert!(check_resumable_complete_state_layout_v2(&bytes[..end],&[7;32]).is_err());
        }
        let mut missing=fields.clone();missing[8].clear();
        assert!(matches!(check_resumable_complete_state_layout_v2(&seal(&mut missing),&[7;32]),Err(Failure::InvalidInput)));
        assert!(matches!(check_resumable_complete_state_layout_v2(&bytes,&[8;32]),Err(Failure::ForeignPesi)));
        // Whole-artifact bound is unchanged, including the integrity slot.
        let mut fit=fields.clone();
        fit[9].resize(1+TOTAL_MAX-bytes.len(),1);
        let exact=seal(&mut fit);assert_eq!(exact.len(),TOTAL_MAX);
        assert!(check_resumable_complete_state_layout_v2(&exact,&[7;32]).is_ok());
        fit[9].push(1);let oversized=seal(&mut fit);
        assert!(matches!(check_resumable_complete_state_layout_v2(&oversized,&[7;32]),Err(Failure::LimitExceeded)));
    }
    #[test]
    fn beta1_reference_complete_frame_admits_only_its_exact_recorded_subject_tuple(){
        use sha2::{Digest,Sha256};
        // Exact SWC2-P02 custody, PESI and type bytes from the sealed R2.1 vector.
        let raw=hex::decode("44444e2d53415645442d574f524c442d4944454e544954592d435553544f44592d5631000001000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f56ef58c78d42e6499fa62157839db76abab11bdf117a1f7e4bf9a6e0917c01b2000000209497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad400000000000000010000000000000001cb6695c2db6786c3d0843d02e5a4cf9172a485206a68aa5a53c351c524a9db6a0000000000000000000000000000000001000000021100").unwrap();
        let pesi:[u8;32]=raw[106..138].try_into().unwrap();let target:[u8;32]=raw[154..186].try_into().unwrap();
        let mut custody=(raw.len() as u64).to_be_bytes().to_vec();custody.extend(&raw);custody.extend(Sha256::digest(&raw));
        let reference=ResourceValue::IdentityReference(ddonirang_core::platform::IdentityReferenceValueV1{
            profile_version:1,target_type_identity:vec![0x11,0],owner_space_identity:raw[70..102].to_vec(),target_identity:target.to_vec(),generation:0,
        }.into());
        let mut leaf=Vec::new();reference.encode_canon(&mut leaf);
        let mut resources=1u64.to_be_bytes().to_vec();resources.extend(1u32.to_be_bytes());resources.push(b'r');resources.extend((leaf.len() as u64).to_be_bytes());resources.extend(&leaf);
        let mut value=Vec::new();ResourceValue::Fixed64(Fixed64::ONE).encode_canon(&mut value);
        let mut subjects=1u64.to_be_bytes().to_vec();subjects.extend(target);subjects.extend((value.len() as u64).to_be_bytes());subjects.extend(value);
        let empty=[0;8];let frame=CompleteStateFrameV1{pesi:&pesi,resources:&resources,parameters:&empty,next_tick:3,subjects:&subjects,atom_evidence:&[],subject_custody:&custody};
        let bytes=frame.encode_canonical().unwrap();let decoded=check_complete_state_frame_v1(&bytes,&pesi).unwrap();
        let values=decoded.decode_value_tables().unwrap();assert_eq!(values.resources["r"],reference);
        assert_eq!(values.subjects[&target].value,ResourceValue::Fixed64(Fixed64::ONE));
        assert_eq!(values.subjects[&target].target_type,[0x11,0]);
        assert_eq!(decoded.encode_canonical().unwrap(),bytes);
        // LIVE identity custody does not substitute for the actual target value.
        let dangling=CompleteStateFrameV1{subjects:&empty,..frame};
        assert!(matches!(dangling.decode_value_tables(),Err(Failure::SubjectCustodyInvalid)));
        assert!(dangling.encode_canonical().is_err());
        // Self-reference is checked once, not followed recursively.
        let mut self_subject=1u64.to_be_bytes().to_vec();self_subject.extend(target);
        self_subject.extend((leaf.len() as u64).to_be_bytes());self_subject.extend(&leaf);
        let cyclic=CompleteStateFrameV1{subjects:&self_subject,..frame};
        assert_eq!(cyclic.decode_value_tables().unwrap().subjects[&target].value,reference);
        assert!(cyclic.encode_canonical().is_ok());
        // Type bytes are preserved, NOT inferred from Fixed64 by this staged owner.
        let missing=CompleteStateFrameV1{subject_custody:&[],..frame};assert!(missing.decode_value_tables().is_err());
        let mut dead=raw.clone();dead[202]=2;let mut dead_custody=(dead.len() as u64).to_be_bytes().to_vec();dead_custody.extend(&dead);dead_custody.extend(Sha256::digest(&dead));
        let dead=CompleteStateFrameV1{subject_custody:&dead_custody,..missing};assert!(dead.decode_value_tables().is_err());
    }
    #[test]
    fn beta1_subject_complete_frame_never_ignores_value_table_suffix_or_missing_custody(){
        let empty=[0;8];let pesi=[7;32];
        let mut suffix=empty.to_vec();suffix.push(0);
        let frame=CompleteStateFrameV1{pesi:&pesi,resources:&empty,parameters:&empty,next_tick:3,
            subjects:&suffix,atom_evidence:&[],subject_custody:&[]};
        assert!(frame.decode_value_tables().is_err());assert!(frame.encode_canonical().is_err());
        let mut subjects=1u64.to_be_bytes().to_vec();subjects.extend([1;32]);
        subjects.extend(1u64.to_be_bytes());subjects.push(0);
        let frame=CompleteStateFrameV1{subjects:&subjects,..frame};
        assert!(matches!(frame.decode_value_tables(),Err(Failure::SubjectCustodyInvalid)));
        assert!(frame.encode_canonical().is_err());
    }
    #[test]
    fn beta1_subject_complete_frame_preserves_zero_reference_custody_and_rejects_foreign_pesi(){
        use sha2::{Digest,Sha256};
        // SWC2-P01 exact bytes, not the old two-byte placeholder.
        let raw=hex::decode("44444e2d53415645442d574f524c442d4944454e544954592d435553544f44592d5631000001000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f56ef58c78d42e6499fa62157839db76abab11bdf117a1f7e4bf9a6e0917c01b2000000209497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad400000000000000000000000000000000").unwrap();
        let pesi:[u8;32]=hex::decode("9497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad4").unwrap().try_into().unwrap();
        let mut custody=(raw.len() as u64).to_be_bytes().to_vec();custody.extend(&raw);custody.extend(Sha256::digest(&raw));
        let empty=[0;8];let frame=CompleteStateFrameV1{pesi:&pesi,resources:&empty,parameters:&empty,next_tick:3,
            subjects:&empty,atom_evidence:&[],subject_custody:&custody};
        let bytes=frame.encode_canonical().unwrap();let checked=check_complete_state_frame_v1(&bytes,&pesi).unwrap();
        let values=checked.decode_value_tables().unwrap();let saved=values.subject_custody.as_ref().unwrap();
        assert_eq!(saved.pesi,pesi);assert_eq!(saved.next_subject_ordinal,0);assert!(saved.rows.is_empty());
        assert_eq!(checked.encode_canonical().unwrap(),bytes);
        let wrong=[0;32];let foreign=CompleteStateFrameV1{pesi:&wrong,..frame};
        assert!(matches!(foreign.decode_value_tables(),Err(Failure::ForeignPesi)));
        let mut stale=raw.clone();stale[106]^=1;
        let mut stale_custody=(stale.len() as u64).to_be_bytes().to_vec();stale_custody.extend(&stale);stale_custody.extend(Sha256::digest(&stale));
        let stale=CompleteStateFrameV1{pesi:&pesi,subject_custody:&stale_custody,..foreign};
        assert!(matches!(stale.decode_value_tables(),Err(Failure::ForeignPesi)));
        let mut suffix=custody.clone();suffix.push(0);
        assert!(super::super::complete_state_tables::decode_saved_world_custody(&suffix,&pesi).is_err());
    }
    #[test]
    fn beta1_atom_complete_frame_selects_its_recorded_contract_not_current_registry(){
        let (raw,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"a",&[vec![1]]).unwrap();
        let atom=ResourceValue::RegisteredAtom(contract.admitted_value(b"a",&[1]).unwrap().into());
        let mut saved=Vec::new();atom.encode_canon(&mut saved);
        let mut resources=1u64.to_be_bytes().to_vec();resources.extend(1u32.to_be_bytes());resources.push(b'a');
        resources.extend((saved.len() as u64).to_be_bytes());resources.extend(saved);
        let mut evidence=1u64.to_be_bytes().to_vec();evidence.extend(contract.custody().snapshot_identity);
        evidence.extend(contract.custody().sha256);evidence.extend((raw.len() as u64).to_be_bytes());evidence.extend(raw);
        let pesi=[7;32];let empty=[0;8];
        let frame=CompleteStateFrameV1{pesi:&pesi,resources:&resources,parameters:&empty,next_tick:3,
            subjects:&empty,atom_evidence:&evidence,subject_custody:&[]};
        let bytes=frame.encode_canonical().unwrap();
        let checked=check_complete_state_frame_v1(&bytes,&pesi).unwrap();
        assert_eq!(checked.decode_value_tables().unwrap().resources["a"],atom);
        assert_eq!(checked.encode_canonical().unwrap(),bytes);
        let omitted=CompleteStateFrameV1{atom_evidence:&[],..frame};
        assert!(omitted.decode_value_tables().is_err());assert!(omitted.encode_canonical().is_err());
        let mut tampered=evidence.clone();*tampered.last_mut().unwrap()^=1;
        let bad=CompleteStateFrameV1{atom_evidence:&tampered,..omitted};
        assert!(bad.decode_value_tables().is_err());assert!(bad.encode_canonical().is_err());
        assert_eq!(resources.len(),checked.resources.len());
    }
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
    fn rd670_complete_frame_common_encoder_matches_independent_exact_bytes() {
        let original=fixture();
        let decoded=check_complete_state_frame_v1(&original,&[7;32]).unwrap();
        assert_eq!(decoded.encode_canonical().unwrap(),original);
        let empty=[0u8;8];
        let parameters=super::super::complete_state_tables::encode_parameter_table(
            &BTreeMap::from([("속도".into(),Fixed64::from_raw_i64(i64::MIN))])).unwrap();
        let frame=CompleteStateFrameV1{pesi:&[7;32],resources:&empty,parameters:&parameters,
            next_tick:u64::MAX,subjects:&empty,atom_evidence:&[],subject_custody:&[]};
        let bytes=frame.encode_canonical().unwrap();
        let decoded=check_complete_state_frame_v1(&bytes,&[7;32]).unwrap();
        assert_eq!(decoded.next_tick,u64::MAX);
        assert_eq!(decoded.decode_value_tables().unwrap().parameters["속도"].raw_i64(),i64::MIN);
        assert_eq!(decoded.encode_canonical().unwrap(),bytes);
    }
    #[test]
    fn rd670_complete_frame_encoder_rejects_invalid_values_and_lengths() {
        let empty=[0u8;8];
        let mut resources=1u64.to_be_bytes().to_vec();
        resources.extend_from_slice(&1u32.to_be_bytes());resources.push(b'a');
        resources.extend_from_slice(&2u64.to_be_bytes());resources.extend_from_slice(&[1,2]);
        let mut frame=CompleteStateFrameV1{pesi:&[7;32],resources:&resources,parameters:&empty,
            next_tick:0,subjects:&empty,atom_evidence:&[],subject_custody:&[]};
        assert!(matches!(frame.encode_canonical(),Err(Failure::InvalidInput)));
        frame.resources=&empty;
        let excessive=vec![0u8;TOTAL_MAX];frame.subject_custody=&excessive;
        assert!(matches!(frame.encode_canonical(),Err(Failure::LimitExceeded)));
        frame.subject_custody=&[];frame.subjects=&[];
        assert!(matches!(frame.encode_canonical(),Err(Failure::InvalidInput)));
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
