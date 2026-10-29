//! WO25 R2 private evidence format. Decoding never grants recovery or execution.
//! Native issuer/witness/full custody admission remains a separate boundary.
use sha2::{Digest,Sha256};
pub(crate) mod witness;

pub(crate) const PROFILE:&str="wo25-finish-recovery-v1";
pub(crate) const HEADER_LIMIT:usize=131_072;
pub(crate) const PAYLOAD_LIMIT:u64=536_870_912;
pub(crate) const CHUNK_BYTES:usize=1_000_000;
pub(crate) const CHUNK_LIMIT:u32=537;
const DOMAIN:&[u8]=b"DDN-WO25-FINISH-RECOVERY-RECORD-V1\0";
const ERROR:&str="E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 마치기 복구 자료의 형식이나 결속이 다릅니다";

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum RecordKindV1 {Intent,ActualFailure}

/// Borrowed structural data only: no public constructor, source/root owner,
/// Running/Finished state, or cold recovery permit can be obtained from it.
pub(crate) struct RecordHeaderV1<'a>{
    raw:&'a [u8],kind:RecordKindV1,fields:[&'a [u8];20],
    payload_length:u64,chunk_count:u32,
}
fn integer(bytes:&[u8])->Result<u64,String>{
    Ok(u64::from_be_bytes(bytes.try_into().map_err(|_|ERROR)?))
}
fn word(bytes:&[u8])->Result<u32,String>{
    Ok(u32::from_be_bytes(bytes.try_into().map_err(|_|ERROR)?))
}
fn native_cause(code:u16)->bool{
    use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1::*;
    [InvalidInput,LimitExceeded,SnapshotHashMismatch,AtomEvidenceInvalid,SubjectCustodyInvalid,
        PesiMismatch,ForeignPesi,StaleRevision,CustodyConflict,ConflictingRetry,UnsupportedDurability,
        WriteFailed,ShortWrite,SyncFailed,RereadFailed,RereadMismatch,AtomicCommitFailed,
        ContainerDurabilityFailed,RevisionCommitFailed,AdapterAborted,Interrupted,
        CorruptRecoveryRecord,RecoveryRequired,ArtifactMismatch,RevisionEnvelopeRequired]
        .into_iter().any(|cause|cause as u16==code)
}
impl<'a> RecordHeaderV1<'a>{
    pub(crate) fn parse(raw:&'a [u8])->Result<Self,String>{
        if raw.len()>HEADER_LIMIT || !raw.starts_with(DOMAIN){return Err(ERROR.into());}
        let prefix=raw.get(DOMAIN.len()..DOMAIN.len()+4).ok_or(ERROR)?;
        if prefix[..2]!=[0,1]{return Err(ERROR.into());}
        let (kind,count)=match (prefix[2],prefix[3]){
            (0,20)=>(RecordKindV1::Intent,20),(1,7)=>(RecordKindV1::ActualFailure,7),
            _=>return Err(ERROR.into()),
        };
        let mut fields=[&[][..];20];let mut offset=DOMAIN.len()+4;
        for (index,field) in fields.iter_mut().take(count).enumerate(){
            let end=offset.checked_add(9).ok_or(ERROR)?;
            let header=raw.get(offset..end).ok_or(ERROR)?;
            if header[0]!=index as u8+1{return Err(ERROR.into());}
            let length=usize::try_from(integer(&header[1..])?).map_err(|_|ERROR)?;
            offset=end;let end=offset.checked_add(length).ok_or(ERROR)?;
            *field=raw.get(offset..end).ok_or(ERROR)?;offset=end;
        }
        if offset!=raw.len(){return Err(ERROR.into());}
        let (length_tag,count_tag)=match kind{
            RecordKindV1::Intent=>{
                let widths=[32,32,32,8,32,8,8,32,0,0,0,0,32,32,8,1,0,32,8,4];
                for (field,width) in fields.iter().zip(widths){
                    if width!=0 && field.len()!=width{return Err(ERROR.into());}
                }
                for tag in [9,10,11,12]{
                    let field=fields[tag-1];if field.is_empty()||field.len()>16384{return Err(ERROR.into());}
                }
                if integer(fields[5])?>i64::MAX as u64 || !(1..=i64::MAX as u64).contains(&integer(fields[6])?)
                    || fields[15][0]>1{return Err(ERROR.into());}
                match fields[16]{
                    [0]=>{},
                    notice if notice.len()==49 && notice[0]==1=>{
                        if integer(&notice[9..17])?.checked_sub(integer(&notice[1..9])?).is_none(){return Err(ERROR.into());}
                    },
                    _=>return Err(ERROR.into()),
                }
                check_lifecycles(&fields)?;(19,20)
            },
            RecordKindV1::ActualFailure=>{
                for (field,width) in fields.iter().zip([32,1,1,2,32,8,4]){
                    if field.len()!=width{return Err(ERROR.into());}
                }
                if !(1..=14).contains(&fields[1][0]){return Err(ERROR.into());}
                let code=u16::from_be_bytes(fields[3].try_into().map_err(|_|ERROR)?);
                match fields[2][0]{0 if code==0=>{},1 if native_cause(code)=>{},_=>return Err(ERROR.into())};
                (6,7)
            },
        };
        let payload_length=integer(fields[length_tag-1])?;
        let chunk_count=word(fields[count_tag-1])?;
        let cap=if kind==RecordKindV1::Intent{PAYLOAD_LIMIT}else{65_536};
        if payload_length==0 || payload_length>cap || chunk_count==0 || chunk_count>CHUNK_LIMIT
            || payload_length.div_ceil(CHUNK_BYTES as u64)!=u64::from(chunk_count){return Err(ERROR.into());}
        Ok(Self{raw,kind,fields,payload_length,chunk_count})
    }
    pub(crate) fn kind(&self)->RecordKindV1{self.kind}
    pub(crate) fn field(&self,tag:u8)->Option<&'a [u8]>{
        let count=if self.kind==RecordKindV1::Intent{20}else{7};
        tag.checked_sub(1).filter(|index|usize::from(*index)<count).map(|index|self.fields[usize::from(index)])
    }
    pub(crate) fn payload_length(&self)->u64{self.payload_length}
    pub(crate) fn chunk_count(&self)->u32{self.chunk_count}
    /// Stream bounded chunks; never allocate the declared full payload.
    /// C01/Tape/summary and all Native custody checks are still required later.
    pub(crate) fn check_payload<B:AsRef<[u8]>>(&self,chunks:impl IntoIterator<Item=B>)->Result<[u8;32],String>{
        let mut payload=Sha256::new();let mut record=Sha256::new();record.update(self.raw);
        let mut count=0u32;let mut length=0u64;
        for chunk in chunks{
            let bytes=chunk.as_ref();
            count=count.checked_add(1).ok_or(ERROR)?;
            if count>self.chunk_count || bytes.is_empty() || bytes.len()>CHUNK_BYTES{return Err(ERROR.into());}
            let expected=if count<self.chunk_count{CHUNK_BYTES as u64}
                else{self.payload_length.checked_sub(u64::from(count-1)*CHUNK_BYTES as u64).ok_or(ERROR)?};
            if bytes.len() as u64!=expected{return Err(ERROR.into());}
            if self.kind==RecordKindV1::ActualFailure{std::str::from_utf8(bytes).map_err(|_|ERROR)?;}
            length=length.checked_add(bytes.len() as u64).ok_or(ERROR)?;payload.update(bytes);record.update(bytes);
        }
        let digest_tag=if self.kind==RecordKindV1::Intent{18}else{5};
        if count!=self.chunk_count || length!=self.payload_length
            || payload.finalize().as_slice()!=self.fields[digest_tag-1]{return Err(ERROR.into());}
        Ok(record.finalize().into())
    }
}
fn check_lifecycles(fields:&[&[u8];20])->Result<(),String>{
    use super::ReplayLifecycleCandidateV1;
    let previous=ReplayLifecycleCandidateV1::parse(fields[8])?;
    let pending=ReplayLifecycleCandidateV1::parse(fields[9])?;
    let ready=ReplayLifecycleCandidateV1::parse(fields[10])?;
    if previous.payload(8)!=Some(&[1][..]) || pending.payload(8)!=Some(&[0][..])
        || ready.payload(8)!=Some(&[1][..]) || pending.payload(9)!=Some(&[1][..]){return Err(ERROR.into());}
    for tag in 1..=17{
        if tag!=8 && pending.payload(tag)!=ready.payload(tag){return Err(ERROR.into());}
    }
    for (header_tag,life_tag) in [(2,1),(3,2),(4,3),(5,5),(6,6),(7,7),(8,10),(13,16),(14,14),(15,15),(16,17)]{
        if Some(fields[header_tag-1])!=pending.payload(life_tag){return Err(ERROR.into());}
    }
    for tag in [1,2,3,4,5,6,10,11,14]{
        if previous.payload(tag)!=pending.payload(tag){return Err(ERROR.into());}
    }
    let old=integer(previous.payload(7).ok_or(ERROR)?)?;
    if old.checked_add(1)!=Some(integer(fields[6])?)
        || Sha256::digest(fields[11]).as_slice()!=pending.payload(12).ok_or(ERROR)?{return Err(ERROR.into());}
    Ok(())
}

/// Canonical structural encoder only. It is not an Intent/Failure issuer.
pub(crate) fn encode_header(kind:RecordKindV1,fields:&[&[u8]])->Result<Vec<u8>,String>{
    let count=if kind==RecordKindV1::Intent{20}else{7};
    if fields.len()!=count{return Err(ERROR.into());}
    let length=fields.iter().try_fold(DOMAIN.len()+4,|size,field|size.checked_add(9)?.checked_add(field.len()))
        .filter(|size|*size<=HEADER_LIMIT).ok_or(ERROR)?;
    let mut out=Vec::new();out.try_reserve_exact(length).map_err(|_|ERROR)?;
    out.extend_from_slice(DOMAIN);out.extend_from_slice(&1u16.to_be_bytes());
    out.push(if kind==RecordKindV1::Intent{0}else{1});out.push(count as u8);
    for (index,field) in fields.iter().enumerate(){out.push(index as u8+1);out.extend_from_slice(&(field.len() as u64).to_be_bytes());out.extend_from_slice(field);}
    RecordHeaderV1::parse(&out)?;Ok(out)
}

#[cfg(test)]
mod tests;
