//! R5 exact protected Grant2 bytes. Decoding never issues a once permit.
//! Grant1 stays byte-identical; only the actual protected owner may write this.
use super::{Failure,GrantRecord,Lifecycle};
const MAGIC:&[u8]=b"DDN-NATIVE-ROOT-GRANT-V2\0";
const MAX:usize=8577;

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
}
impl Grant2 {
    fn validate(&self)->Result<(),Failure>{
        if self.initial.revision.is_empty()||self.initial.revision.len()>4096 {
            return Err(Failure::CorruptRecoveryRecord);
        }
        if let FirstUse::Started(parent)=&self.state {
            if parent.revision.is_empty()||parent.revision.len()>4096{return Err(Failure::CorruptRecoveryRecord);}
        }
        Ok(())
    }
    pub(super) fn encode(&self)->Result<Vec<u8>,Failure>{
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
            integrity:fields[12].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?},state};
        record.validate()?;Ok(record)
    }
    /// Candidate bytes only. No permission is returned until real protected
    /// write/flush/fresh readback and root/source custody have all succeeded.
    pub(super) fn started_candidate(&self,parent:SourceAnchor)->Result<Self,Failure>{
        if self.base.lifecycle!=Lifecycle::Active||self.state!=FirstUse::Unused {
            return Err(Failure::RecoveryRequired);
        }
        let candidate=Self{base:self.base.clone(),initial:self.initial.clone(),state:FirstUse::Started(parent)};
        candidate.validate()?;Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn original()->Grant2{Grant2{base:GrantRecord::decode(include_bytes!("../root_grant_fixture.bin")).unwrap(),
        initial:SourceAnchor{revision:vec![1],integrity:[2;32]},state:FirstUse::Unused}}
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
}
