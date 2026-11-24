//! R5 exact protected Grant2 bytes. Decoding never issues a once permit.
//! Grant1 stays byte-identical; only the actual protected owner may write this.
use super::{Failure,GrantRecord,Lifecycle};
const MAGIC:&[u8]=b"DDN-NATIVE-ROOT-GRANT-V2\0";
const MAX:usize=8577;
// Adopted first-failure recovery is a distinct protected-record revision.
// Every entry retains a hash of the entire preceding Started record and a
// source-bound Aborted/no-world witness. Decoding issues no retry capability.
const RECOVERY_MAGIC:&[u8]=b"DDN-NATIVE-ROOT-GRANT-V3\0";
const MAX_RETRIES:usize=8;

#[derive(Clone,PartialEq,Eq)]
pub(super) struct SourceAnchor {
    pub(super) revision:Vec<u8>,
    pub(super) integrity:[u8;32],
}
#[derive(Clone,PartialEq,Eq)]
pub(super) enum FirstUse {Unused,Started(SourceAnchor)}
#[derive(Clone,PartialEq,Eq)]
pub(super) struct Grant2 {
    pub(super) base:GrantRecord,
    pub(super) initial:SourceAnchor,
    pub(super) state:FirstUse,
    pub(super) retry_history:Vec<[u8;64]>,
}
impl Grant2 {
    fn validate(&self)->Result<(),Failure>{
        if self.initial.revision.is_empty()||self.initial.revision.len()>4096 {
            return Err(Failure::CorruptRecoveryRecord);
        }
        if let FirstUse::Started(parent)=&self.state {
            if parent.revision.is_empty()||parent.revision.len()>4096{return Err(Failure::CorruptRecoveryRecord);}
        }
        if self.retry_history.len()>MAX_RETRIES
            ||(!self.retry_history.is_empty()&&!matches!(self.state,FirstUse::Started(_))){
            return Err(Failure::CorruptRecoveryRecord);
        }
        Ok(())
    }
    pub(super) fn encode(&self)->Result<Vec<u8>,Failure>{
        let legacy=self.encode_legacy()?;
        if self.retry_history.is_empty(){return Ok(legacy);}
        let mut bytes=Vec::with_capacity(legacy.len()+RECOVERY_MAGIC.len()+8+64*self.retry_history.len());
        bytes.extend(RECOVERY_MAGIC);bytes.extend([0,3]);
        bytes.extend((legacy.len() as u32).to_be_bytes());bytes.extend(legacy);
        bytes.extend((self.retry_history.len() as u16).to_be_bytes());
        for entry in &self.retry_history{bytes.extend(entry);}
        Ok(bytes)
    }
    fn encode_legacy(&self)->Result<Vec<u8>,Failure>{
        self.validate()?;
        let base=self.base.encode()?;
        let mut bytes=Vec::new();bytes.try_reserve_exact(MAX).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend(MAGIC);bytes.extend([0,2,15]);
        bytes.extend(&base[super::MAGIC.len()+3..]);
        let (state,parent,integrity)=match &self.state {
            FirstUse::Unused=>([0],&[][..],&[0u8;32]),
            FirstUse::Started(parent)=>([1],parent.revision.as_slice(),&parent.integrity),
        };
        let fields:[&[u8];5]=[&state,&self.initial.revision,&self.initial.integrity,parent,integrity];
        for (index,field) in fields.iter().enumerate(){bytes.push(11+index as u8);bytes.extend((field.len() as u32).to_be_bytes());bytes.extend(*field);}
        Ok(bytes)
    }
    pub(super) fn decode(bytes:&[u8])->Result<Self,Failure>{
        if bytes.starts_with(RECOVERY_MAGIC){return Self::decode_recovery(bytes);}
        if bytes.len()>MAX||!bytes.starts_with(MAGIC)||bytes.get(MAGIC.len()..MAGIC.len()+3)!=Some(&[0,2,15]) {
            return Err(Failure::CorruptRecoveryRecord);
        }
        let mut cursor=MAGIC.len()+3;
        let old_start=cursor;let mut old_end=cursor;
        let mut fields:[&[u8];15]=[&[];15];
        for (index,field) in fields.iter_mut().enumerate(){
            if bytes.get(cursor)!=Some(&(index as u8+1)){return Err(Failure::CorruptRecoveryRecord);}
            let raw=bytes.get(cursor+1..cursor+5).ok_or(Failure::CorruptRecoveryRecord)?;
            let size=u32::from_be_bytes(raw.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize;
            let valid=match index {
                0..=9=>size==super::LENGTHS[index],
                10=>size==1,11=>size>0&&size<=4096,12|14=>size==32,13=>size<=4096,_=>false,
            };
            if !valid{return Err(Failure::CorruptRecoveryRecord);}
            cursor+=5;
            *field=bytes.get(cursor..cursor+size).ok_or(Failure::CorruptRecoveryRecord)?;
            cursor+=size;if index==9{old_end=cursor;}
        }
        if cursor!=bytes.len(){return Err(Failure::CorruptRecoveryRecord);}
        let mut old=Vec::with_capacity(super::PLAINTEXT_BYTES);
        old.extend(super::MAGIC);old.extend([0,1,10]);old.extend(&bytes[old_start..old_end]);
        let base=GrantRecord::decode(&old)?;old.fill(0);
        let state=match fields[10] {
            [0] if fields[13].is_empty()&&fields[14]==[0;32]=>FirstUse::Unused,
            [1] if !fields[13].is_empty()=>FirstUse::Started(SourceAnchor{
                revision:fields[13].to_vec(),integrity:fields[14].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?}),
            _=>return Err(Failure::CorruptRecoveryRecord),
        };
        let record=Self{base,initial:SourceAnchor{revision:fields[11].to_vec(),
            integrity:fields[12].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?},state,retry_history:Vec::new()};
        record.validate()?;Ok(record)
    }
    /// Candidate bytes only. No permission is returned until real protected
    /// write/flush/fresh readback and root/source custody have all succeeded.
    pub(super) fn started_candidate(&self,parent:SourceAnchor)->Result<Self,Failure>{
        if self.base.lifecycle!=Lifecycle::Active||self.state!=FirstUse::Unused {
            return Err(Failure::RecoveryRequired);
        }
        let candidate=Self{base:self.base.clone(),initial:self.initial.clone(),state:FirstUse::Started(parent),retry_history:Vec::new()};
        candidate.validate()?;Ok(candidate)
    }
    fn aborted_entry(&self)->Result<[u8;64],Failure>{
        use sha2::{Digest,Sha256};
        let FirstUse::Started(source)=&self.state else{return Err(Failure::RecoveryRequired)};
        let prior:[u8;32]=Sha256::digest(self.encode()?).into();
        let mut proof=Sha256::new();proof.update(b"ddn.native.aborted-preworld.owner.v1\0");
        proof.update(prior);proof.update(&source.revision);proof.update(source.integrity);
        proof.update(self.initial.integrity);proof.update(self.base.registration_pesi);
        let mut entry=[0;64];entry[..32].copy_from_slice(&prior);entry[32..].copy_from_slice(&proof.finalize());Ok(entry)
    }
    /// Candidate only. The protected owner must prove the complete pinned
    /// source-only history, no world/custody rows and no live attempt under its
    /// exclusive lease immediately before CAS/flush/fresh readback.
    pub(super) fn retry_candidate(&self)->Result<Self,Failure>{
        if self.base.lifecycle!=Lifecycle::Active||self.retry_history.len()>=MAX_RETRIES{return Err(Failure::RecoveryRequired);}
        let mut next=self.clone();next.retry_history.push(self.aborted_entry()?);next.validate()?;Ok(next)
    }
    fn decode_recovery(bytes:&[u8])->Result<Self,Failure>{
        let header=RECOVERY_MAGIC.len();
        if bytes.len()>MAX+header+8+64*MAX_RETRIES||bytes.get(header..header+2)!=Some(&[0,3]){return Err(Failure::CorruptRecoveryRecord);}
        let size=u32::from_be_bytes(bytes.get(header+2..header+6).ok_or(Failure::CorruptRecoveryRecord)?.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize;
        if size>MAX{return Err(Failure::CorruptRecoveryRecord);}
        let end=(header+6).checked_add(size).ok_or(Failure::CorruptRecoveryRecord)?;
        let legacy=bytes.get(header+6..end).ok_or(Failure::CorruptRecoveryRecord)?;
        if !legacy.starts_with(MAGIC){return Err(Failure::CorruptRecoveryRecord);}
        let mut record=Self::decode(legacy)?;
        let count=u16::from_be_bytes(bytes.get(end..end+2).ok_or(Failure::CorruptRecoveryRecord)?.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize;
        if count==0||count>MAX_RETRIES||bytes.len()!=end+2+count*64{return Err(Failure::CorruptRecoveryRecord);}
        for entry in bytes[end+2..].chunks_exact(64){
            let expected=record.aborted_entry()?;
            if entry!=expected{return Err(Failure::CorruptRecoveryRecord);}
            record.retry_history.push(expected);
        }
        record.validate()?;Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn original()->Grant2{Grant2{base:GrantRecord::decode(include_bytes!("../root_grant_fixture.bin")).unwrap(),
        initial:SourceAnchor{revision:vec![1],integrity:[2;32]},state:FirstUse::Unused,retry_history:Vec::new()}}
    #[test]
    fn vc04_grant2_exact_codec_preserves_old_base_and_once_state(){
        let original=original();let old=original.base.encode().unwrap();
        let bytes=original.encode().unwrap();assert!(Grant2::decode(&bytes).unwrap()==original);
        assert!(GrantRecord::decode(&bytes).is_err());assert_eq!(original.base.encode().unwrap(),old);
        let started=original.started_candidate(SourceAnchor{revision:vec![3],integrity:[4;32]}).unwrap();
        assert!(Grant2::decode(&started.encode().unwrap()).unwrap()==started);
        assert!(started.started_candidate(SourceAnchor{revision:vec![3],integrity:[4;32]}).is_err());
        let mut maximum=original;maximum.initial.revision=vec![1;4096];
        maximum.state=FirstUse::Started(SourceAnchor{revision:vec![3;4096],integrity:[4;32]});
        let raw=maximum.encode().unwrap();assert_eq!(raw.len(),MAX);assert!(Grant2::decode(&raw).is_ok());
    }
    #[test]
    fn vc04_grant2_rejects_truncation_order_state_and_legacy_aliases(){
        let bytes=original().encode().unwrap();
        for end in 0..bytes.len(){assert!(Grant2::decode(&bytes[..end]).is_err(),"truncation {end}");}
        for offset in [MAGIC.len()+1,MAGIC.len()+2,MAGIC.len()+3,super::super::PLAINTEXT_BYTES] {
            let mut bad=bytes.clone();bad[offset]^=0xff;assert!(Grant2::decode(&bad).is_err(),"offset {offset}");
        }
        let mut extra=bytes.clone();extra.push(0);assert!(Grant2::decode(&extra).is_err());
        assert!(Grant2::decode(include_bytes!("../root_grant_fixture.bin")).is_err());
        let mut too_big=original();too_big.initial.revision=vec![1;4097];assert!(too_big.encode().is_err());
    }
    #[test]
    fn protected_retry_record_retains_started_chain_and_rejects_unknown_or_corrupt_history(){
        let started=original().started_candidate(SourceAnchor{revision:vec![3],integrity:[4;32]}).unwrap();
        let old=started.encode().unwrap();let mut retry=started.retry_candidate().unwrap();
        assert!(retry.state==started.state);assert!(retry.initial==started.initial);
        assert_eq!(retry.encode_legacy().unwrap(),old);
        for _ in 1..MAX_RETRIES{retry=retry.retry_candidate().unwrap();}
        let bytes=retry.encode().unwrap();assert!(Grant2::decode(&bytes).unwrap()==retry);
        assert!(retry.retry_candidate().is_err());assert!(original().retry_candidate().is_err());
        assert!(!bytes.starts_with(MAGIC));assert!(GrantRecord::decode(&bytes).is_err());
        for end in 0..bytes.len(){assert!(Grant2::decode(&bytes[..end]).is_err(),"truncated {end}");}
        for index in [RECOVERY_MAGIC.len()+1,bytes.len()-1,bytes.len()-64]{let mut bad=bytes.clone();bad[index]^=1;assert!(Grant2::decode(&bad).is_err());}
        let mut bad=bytes.clone();bad.push(0);assert!(Grant2::decode(&bad).is_err());
    }
}
