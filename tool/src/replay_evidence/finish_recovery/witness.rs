//! Structural witness plaintext only. DPAPI/current/lease admission is Native.
use super::{integer,RecordHeaderV1,RecordKindV1,ERROR};
use sha2::{Digest,Sha256};
const DOMAIN:&[u8]=b"DDN-WO25-FINISH-RECOVERY-WITNESS-V1\0";
const WIDTHS:[usize;12]=[32,32,8,32,8,8,32,32,32,1,32,32];

pub(crate) struct WitnessCandidateV1<'a>{raw:&'a [u8],fields:[&'a [u8];12]}
impl<'a> WitnessCandidateV1<'a>{
    pub(crate) fn parse(raw:&'a [u8])->Result<Self,String>{
        let expected=DOMAIN.len()+3+12*9+WIDTHS.iter().sum::<usize>();
        if raw.len()!=expected || !raw.starts_with(DOMAIN)
            || raw.get(DOMAIN.len()..DOMAIN.len()+3)!=Some(&[0,1,12][..]){return Err(ERROR.into());}
        let mut fields=[&[][..];12];let mut offset=DOMAIN.len()+3;
        for (index,width) in WIDTHS.into_iter().enumerate(){
            let header=raw.get(offset..offset+9).ok_or(ERROR)?;
            if header[0]!=index as u8+1 || integer(&header[1..])?!=width as u64{return Err(ERROR.into());}
            offset+=9;fields[index]=raw.get(offset..offset+width).ok_or(ERROR)?;offset+=width;
        }
        let candidate=Self{raw,fields};let zero=[0;32];
        let stage=fields[9][0];
        if integer(fields[4])?>i64::MAX as u64 || !(1..=i64::MAX as u64).contains(&integer(fields[5])?)
            || stage>2 || (stage<2 && fields[8]!=zero) || (stage==2 && fields[8]==zero){return Err(ERROR.into());}
        if fields[10]!=candidate.expected_previous()?.as_slice(){return Err(ERROR.into());}
        Ok(candidate)
    }
    fn expected_previous(&self)->Result<[u8;32],String>{
        if self.fields[9]==[0]{return Ok([0;32]);}
        let zero=[0;32];let initial_stage=[0];let middle_stage=[1];
        let mut initial:[&[u8];12]=self.fields;
        initial[8]=&zero;initial[9]=&initial_stage;initial[10]=&zero;
        let first_hash:[u8;32]=Sha256::digest(encode_raw(&initial)?).into();
        if self.fields[9]==[1]{return Ok(first_hash);}
        initial[9]=&middle_stage;initial[10]=&first_hash;
        Ok(Sha256::digest(encode_raw(&initial)?).into())
    }
    pub(crate) fn stage(&self)->u8{self.fields[9][0]}
    pub(crate) fn raw(&self)->&'a [u8]{self.raw}
    pub(crate) fn field(&self,tag:u8)->Option<&'a [u8]>{
        tag.checked_sub(1).and_then(|index|self.fields.get(usize::from(index))).copied()
    }
    pub(crate) fn check_intent(&self,intent:&RecordHeaderV1<'_>,record_digest:&[u8;32])->Result<(),String>{
        if intent.kind()!=RecordKindV1::Intent || self.fields[7]!=record_digest{return Err(ERROR.into());}
        for (witness_tag,intent_tag) in [(1,2),(2,3),(3,4),(4,5),(5,6),(6,7),(7,8)]{
            if Some(self.fields[witness_tag-1])!=intent.field(intent_tag){return Err(ERROR.into());}
        }
        if self.fields[11]!=Sha256::digest(intent.field(10).ok_or(ERROR)?).as_slice(){return Err(ERROR.into());}
        Ok(())
    }
}
fn encode_raw(fields:&[&[u8];12])->Result<Vec<u8>,String>{
    for (field,width) in fields.iter().zip(WIDTHS){if field.len()!=width{return Err(ERROR.into());}}
    let length=DOMAIN.len()+3+12*9+WIDTHS.iter().sum::<usize>();
    let mut raw=Vec::new();raw.try_reserve_exact(length).map_err(|_|ERROR)?;
    raw.extend_from_slice(DOMAIN);raw.extend_from_slice(&[0,1,12]);
    for (index,field) in fields.iter().enumerate(){
        raw.push(index as u8+1);raw.extend_from_slice(&(field.len() as u64).to_be_bytes());raw.extend_from_slice(field);
    }Ok(raw)
}
/// Encoding data is not protecting it or admitting a Native recovery request.
pub(crate) fn encode(fields:&[&[u8];12])->Result<Vec<u8>,String>{
    let raw=encode_raw(fields)?;WitnessCandidateV1::parse(&raw)?;Ok(raw)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn wo25_finish_witness_structural_chain_cannot_skip_its_previous_plaintext(){
        let zero=[0;32];let one=1u64.to_be_bytes();let epoch=0u64.to_be_bytes();
        let intent=[8;32];let failure=[9;32];let stage0=[0];let stage1=[1];let stage2=[2];
        let mut fields:[&[u8];12]=[&zero,&zero,&one,&zero,&epoch,&one,&zero,&intent,&zero,&stage0,&zero,&zero];
        let initial=encode(&fields).unwrap();let first_hash=Sha256::digest(&initial);
        fields[9]=&stage1;fields[10]=&first_hash;
        let middle=encode(&fields).unwrap();let middle_hash=Sha256::digest(&middle);
        fields[8]=&failure;fields[9]=&stage2;fields[10]=&middle_hash;
        let final_bytes=encode(&fields).unwrap();
        let checked=WitnessCandidateV1::parse(&final_bytes).unwrap();
        assert_eq!(checked.stage(),2);assert_eq!(checked.raw(),final_bytes);
        fields[10]=&first_hash;
        assert!(encode(&fields).is_err(),"stage2 cannot pretend to follow stage0");
    }
}
