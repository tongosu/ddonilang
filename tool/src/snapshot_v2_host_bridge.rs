//! RD-670 transport admission under the common Rust codec/transaction owner.
//! Raw request fields are comparison evidence, never semantic identity supply.
//! This gate does not publish a session, receipt or restored world.

use sha2::{Digest,Sha256};
use crate::snapshot_v2_durable_transaction::{TransactionBindingV1,
    SnapshotV2DurabilityFailureV1 as Failure};
use crate::unit_wire::{check_complete_state_frame_v1,CompleteStateFrameV1,CompleteStateValueTablesV1};

const DOMAIN:&[u8]=b"DDN-SNAPSHOT-V2-HOST-BRIDGE-REQUEST-V1\0";
// 2 MB complete state plus bounded identity fields and fixed transport fields.
// No adapter may relax the independently checked complete-state 2 MB ceiling.
const REQUEST_MAX:usize=2_030_000;

#[derive(Debug)]
pub(crate) struct SnapshotV2RequestFrameV1<'a> {
    pub(crate) fields:[&'a [u8];13],
    pub(crate) snapshot:CompleteStateFrameV1<'a>,
    pub(crate) values:CompleteStateValueTablesV1,
    pub(crate) request_kind:u8,
    pub(crate) transaction_digest:[u8;32],
}

struct Cursor<'a>{bytes:&'a [u8],offset:usize}
impl<'a> Cursor<'a>{
    fn take(&mut self,count:usize)->Result<&'a [u8],Failure>{
        let end=self.offset.checked_add(count).ok_or(Failure::LimitExceeded)?;
        let value=self.bytes.get(self.offset..end).ok_or(Failure::InvalidInput)?;
        self.offset=end;Ok(value)
    }
    fn field(&mut self,tag:u8)->Result<&'a [u8],Failure>{
        if self.take(1)?!=[tag]{return Err(Failure::InvalidInput);}
        let length=u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
        let length=usize::try_from(length).map_err(|_|Failure::LimitExceeded)?;
        self.take(length)
    }
    fn finish(&self)->Result<(),Failure>{
        if self.offset==self.bytes.len(){Ok(())}else{Err(Failure::InvalidInput)}
    }
}
fn fixed32(bytes:&[u8])->Result<[u8;32],Failure>{bytes.try_into().map_err(|_|Failure::InvalidInput)}
fn hash(bytes:&[u8])->[u8;32]{Sha256::digest(bytes).into()}

/// Validate transport and all duplicated hash/length comparisons without
/// copying the snapshot or trusting host-precomputed checksums. Resource and
/// custody typed validation follows this borrowed gate before owner prepare.
pub(crate) fn check_snapshot_v2_request_frame_v1<'a>(
    raw:&'a [u8],admitted_pesi:&[u8;32],
)->Result<SnapshotV2RequestFrameV1<'a>,Failure>{
    if raw.len()>REQUEST_MAX{return Err(Failure::LimitExceeded);}
    let mut outer=Cursor{bytes:raw,offset:0};
    if outer.take(DOMAIN.len())?!=DOMAIN||outer.take(2)?!=[0,1]{return Err(Failure::InvalidInput);}
    let length=u64::from_be_bytes(outer.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?);
    let length=usize::try_from(length).map_err(|_|Failure::LimitExceeded)?;
    if length>REQUEST_MAX{return Err(Failure::LimitExceeded);}
    let digest=fixed32(outer.take(32)?)?;
    let body=outer.take(length)?;outer.finish()?;
    if hash(body)!=digest{return Err(Failure::InvalidInput);}
    let mut cursor=Cursor{bytes:body,offset:0};
    if cursor.take(1)?!=[13]{return Err(Failure::InvalidInput);}
    let mut fields=[&[][..];13];
    for (index,field) in fields.iter_mut().enumerate(){*field=cursor.field(index as u8+1)?;}
    cursor.finish()?;
    for index in [0,1,2,3]{
        if fields[index].len()>4096{return Err(Failure::LimitExceeded);}
        if index!=1&&fields[index].is_empty(){return Err(Failure::InvalidInput);}
    }
    for index in [4,5,7,8,9,10]{fixed32(fields[index])?;}
    if fields[6].len()!=8||fields[12].len()!=1||!matches!(fields[12][0],1..=3){return Err(Failure::InvalidInput);}
    let snapshot_length=u64::from_be_bytes(fields[6].try_into().map_err(|_|Failure::InvalidInput)?);
    if snapshot_length!=fields[11].len() as u64||hash(fields[11])!=fixed32(fields[5])?{
        return Err(Failure::SnapshotHashMismatch);
    }
    if fields[4]!=admitted_pesi{return Err(Failure::ForeignPesi);}
    let snapshot=check_complete_state_frame_v1(fields[11],admitted_pesi)?;
    if hash(snapshot.atom_evidence)!=fixed32(fields[8])?{return Err(Failure::AtomEvidenceInvalid);}
    if hash(snapshot.subject_custody)!=fixed32(fields[9])?{return Err(Failure::SubjectCustodyInvalid);}
    let binding=TransactionBindingV1{project:fields[0].to_vec(),expected_parent:fields[1].to_vec(),
        new_revision:fields[2].to_vec(),execution_profile:fields[3].to_vec(),
        pesi:*admitted_pesi,snapshot_sha256:fixed32(fields[5])?,snapshot_length,
        revision_integrity:fixed32(fields[7])?};
    let transaction_digest=binding.transaction_digest()?;
    if transaction_digest!=fixed32(fields[10])?{return Err(Failure::CustodyConflict);}
    let values=snapshot.decode_value_tables()?;
    Ok(SnapshotV2RequestFrameV1{fields,snapshot,values,request_kind:fields[12][0],transaction_digest})
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->Vec<u8>{
        fixture_with_resources(&[0u8;8])
    }
    fn fixture_with_resources(resources:&[u8])->Vec<u8>{
        let mut snapshot=b"DDN-SNAPSHOT-V2-COMPLETE-STATE-V1\0".to_vec();snapshot.extend([0,1,8]);
        let pesi=[7u8;32];let empty=[0u8;8];let tick=3u64.to_be_bytes();
        for (index,value) in [&pesi[..],resources,&empty,&tick,&empty,&[][..],&[][..],&[0][..]].iter().enumerate(){
            snapshot.push(index as u8+1);snapshot.extend((value.len() as u64).to_be_bytes());snapshot.extend(*value);
        }
        let binding=TransactionBindingV1{project:b"project".to_vec(),expected_parent:vec![],new_revision:b"revision".to_vec(),
            execution_profile:b"profile".to_vec(),pesi,snapshot_sha256:hash(&snapshot),snapshot_length:snapshot.len() as u64,
            revision_integrity:[8;32]};
        let length=binding.snapshot_length.to_be_bytes();let empty_digest=hash(&[]);let tx=binding.transaction_digest().unwrap();
        let fields=[&binding.project[..],&binding.expected_parent,&binding.new_revision,&binding.execution_profile,
            &pesi,&binding.snapshot_sha256,&length,&binding.revision_integrity,&empty_digest,&empty_digest,&tx,&snapshot,&[1][..]];
        let mut body=vec![13];for (index,value) in fields.iter().enumerate(){body.push(index as u8+1);body.extend((value.len() as u32).to_be_bytes());body.extend(*value);}
        wrap(&body)
    }
    fn wrap(body:&[u8])->Vec<u8>{
        let mut raw=DOMAIN.to_vec();raw.extend([0,1]);raw.extend((body.len() as u64).to_be_bytes());raw.extend(hash(body));raw.extend(body);raw
    }
    #[test]
    fn rd670_request_checks_all_comparison_fields_without_receipt(){
        let raw=fixture();let request=check_snapshot_v2_request_frame_v1(&raw,&[7;32]).unwrap();
        assert_eq!(request.request_kind,1);assert_eq!(request.snapshot.next_tick,3);
        assert!(request.values.resources.is_empty());assert!(request.values.parameters.is_empty());
        assert_eq!(request.fields[0],b"project");assert_eq!(request.transaction_digest,fixed32(request.fields[10]).unwrap());
        assert!(matches!(check_snapshot_v2_request_frame_v1(&raw,&[9;32]),Err(Failure::ForeignPesi)));
        let offset=DOMAIN.len()+2+8+32;
        for index in [5,8,9,10]{
            let mut body=raw[offset..].to_vec();let request=check_snapshot_v2_request_frame_v1(&raw,&[7;32]).unwrap();
            let field_offset=request.fields[index].as_ptr() as usize-raw.as_ptr() as usize-offset;
            body[field_offset]^=1;assert!(check_snapshot_v2_request_frame_v1(&wrap(&body),&[7;32]).is_err());
        }
        let mut bad=raw.clone();bad.push(0);assert!(check_snapshot_v2_request_frame_v1(&bad,&[7;32]).is_err());
        let mut bad=raw;*bad.last_mut().unwrap()^=1;assert!(check_snapshot_v2_request_frame_v1(&bad,&[7;32]).is_err());
    }
    #[test]
    fn rd670_request_rejects_invalid_value_even_with_recomputed_transport_hashes(){
        let mut table=1u64.to_be_bytes().to_vec();table.extend(1u32.to_be_bytes());table.push(b'a');
        table.extend(2u64.to_be_bytes());table.extend([1,1]);
        let raw=fixture_with_resources(&table);
        let request=check_snapshot_v2_request_frame_v1(&raw,&[7;32]).unwrap();
        assert_eq!(request.values.resources["a"],ddonirang_core::platform::ResourceValue::Bool(true));
        *table.last_mut().unwrap()=2;
        // Fixture recomputes snapshot digest, transaction digest and transport
        // checksum: integrity matches, but typed Boolean still must fail.
        assert!(matches!(check_snapshot_v2_request_frame_v1(&fixture_with_resources(&table),&[7;32]),Err(Failure::InvalidInput)));
    }
}
