//! Execution-only lifecycle; deliberately contains no saved PESI or permit.
use super::*;
use ddonirang_core::platform::IdentityReferenceValueV1;
const HEADER:&[u8]=b"DDN-VC04-TEMPORARY-CUSTODY-V1\0";
#[derive(Clone)]
pub(crate) struct TemporarySubjectOwnerV1 {
    context:[u8;32],nonce:[u8;32],world:[u8;32],next:u64,rows:Vec<SavedSubjectRow>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vc04_temporary_custody_actual_lifecycle_mutation_and_saved_firewall(){
        let mut context=[0;32];getrandom::fill(&mut context).unwrap();
        let mut owner=TemporarySubjectOwnerV1::fresh(context).unwrap();
        let empty=owner.encode().unwrap();
        let first=owner.allocate(&[0x42;32]).unwrap();
        let live=owner.encode().unwrap();assert_ne!(live,empty);
        owner.references().unwrap().verify(&first).unwrap();
        owner.retire(&first).unwrap();let dead=owner.encode().unwrap();assert_ne!(live,dead);
        assert!(owner.retire(&first).is_err());
        let second=owner.allocate(&[0x42;32]).unwrap();assert_ne!(first.target_identity,second.target_identity);
        let good=owner.encode().unwrap();assert_eq!(owner.restage(&good).unwrap().encode().unwrap(),good);
        for kind in 0..3{
            let mut bad=good.clone();match kind{0=>bad.push(0),1=>bad[HEADER.len()+1]=2,_=>bad[HEADER.len()+2]^=1};
            assert!(owner.restage(&bad).is_err());
            assert_eq!(owner.restage(&good).unwrap().encode().unwrap(),good,"exact restore GREEN");
        }
        assert!(SavedWorldIdentityCustodyV1::load_recorded(&good,good.len() as u64,Sha256::digest(&good).into(),&[0;32]).is_err());
        let foreign=TemporarySubjectOwnerV1::fresh(context).unwrap();assert!(foreign.restage(&good).is_err());
        assert!(foreign.references().unwrap().verify(&second).is_err());
    }
}
impl TemporarySubjectOwnerV1 {
    pub(crate) fn fresh(context:[u8;32])->Result<Self,Failure>{
        let mut nonce=[0;32];getrandom::fill(&mut nonce).map_err(|_|Failure::SubjectCustodyInvalid)?;
        Ok(Self{context,nonce,world:recorded_world_identity(&nonce)?,next:0,rows:Vec::new()})
    }
    pub(crate) fn world(&self)->&[u8;32]{&self.world}
    pub(crate) fn rows(&self)->&[SavedSubjectRow]{&self.rows}
    pub(crate) fn references(&self)->Result<SubjectReferenceContextV1<'_>,Failure>{SubjectReferenceContextV1::from_lifecycle_parts(&self.world,&self.rows)}
    pub(crate) fn allocate(&mut self,target_type:&[u8])->Result<IdentityReferenceValueV1,Failure>{
        if target_type.is_empty()||target_type.len()>4096{return Err(Failure::SubjectCustodyInvalid)}
        if self.next==u64::MAX||self.rows.len()>=65536{return Err(Failure::LimitExceeded)}
        self.rows.try_reserve(1).map_err(|_|Failure::LimitExceeded)?;
        let id=recorded_subject_identity(&self.world,self.next);
        let reference=IdentityReferenceValueV1{profile_version:1,target_type_identity:target_type.to_vec(),owner_space_identity:self.world.to_vec(),target_identity:id.to_vec(),generation:0};
        self.rows.push(SavedSubjectRow{identity:id,ordinal:self.next,live:true,target_type:target_type.to_vec()});self.next+=1;
        Ok(reference)
    }
    pub(crate) fn retire(&mut self,reference:&IdentityReferenceValueV1)->Result<(),Failure>{
        self.references()?.verify(reference)?;
        self.rows.iter_mut().find(|row|row.identity.as_slice()==reference.target_identity).ok_or(Failure::SubjectCustodyInvalid)?.live=false;Ok(())
    }
    pub(crate) fn encode(&self)->Result<Vec<u8>,Failure>{
        if self.rows.len()>65536||self.next!=self.rows.len() as u64||recorded_world_identity(&self.nonce)?!=self.world{return Err(Failure::SubjectCustodyInvalid)}
        let mut length=HEADER.len()+2+96+16;
        for (index,row) in self.rows.iter().enumerate(){
            if row.ordinal!=index as u64||row.identity!=recorded_subject_identity(&self.world,row.ordinal)||row.target_type.is_empty()||row.target_type.len()>4096{return Err(Failure::SubjectCustodyInvalid)}
            length=length.checked_add(45+row.target_type.len()).filter(|n|*n<=1_000_000).ok_or(Failure::LimitExceeded)?;
        }
        let mut bytes=Vec::new();bytes.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend(HEADER);bytes.extend(1u16.to_be_bytes());bytes.extend(self.context);bytes.extend(self.nonce);bytes.extend(self.world);
        bytes.extend(self.next.to_be_bytes());bytes.extend((self.rows.len() as u64).to_be_bytes());
        let mut rows=self.rows.iter().collect::<Vec<_>>();rows.sort_by_key(|r|r.identity);
        for row in rows{bytes.extend(row.identity);bytes.extend(row.ordinal.to_be_bytes());bytes.push(u8::from(row.live));bytes.extend((row.target_type.len() as u32).to_be_bytes());bytes.extend(&row.target_type);}
        Ok(bytes)
    }
    /// Restage only against this invocation's retained actual context/nonce.
    /// Arbitrary well-formed bytes cannot construct an execution owner.
    pub(crate) fn restage(&self,bytes:&[u8])->Result<Self,Failure>{
        if bytes.len()>1_000_000{return Err(Failure::LimitExceeded)}
        let mut r=Cursor{bytes,at:0};
        if r.take(HEADER.len())?!=HEADER||r.take(2)?!=1u16.to_be_bytes()||r.take(32)?!=self.context||r.take(32)?!=self.nonce||r.take(32)?!=self.world{return Err(Failure::SubjectCustodyInvalid)}
        let next=u64::from_be_bytes(r.take(8)?.try_into().unwrap());let count=u64::from_be_bytes(r.take(8)?.try_into().unwrap());
        if count>65536||next!=count||count>(bytes.len().saturating_sub(r.at)/46) as u64{return Err(Failure::LimitExceeded)}
        let mut rows=Vec::new();rows.try_reserve_exact(count as usize).map_err(|_|Failure::LimitExceeded)?;
        let mut previous=None;
        for _ in 0..count{
            let identity:[u8;32]=r.take(32)?.try_into().unwrap();
            if previous.is_some_and(|old|old>=identity){return Err(Failure::SubjectCustodyInvalid)}previous=Some(identity);
            let ordinal=u64::from_be_bytes(r.take(8)?.try_into().unwrap());
            let live=match r.take(1)?[0]{0=>false,1=>true,_=>return Err(Failure::SubjectCustodyInvalid)};
            let n=u32::from_be_bytes(r.take(4)?.try_into().unwrap()) as usize;
            if n==0||n>4096{return Err(Failure::LimitExceeded)}
            rows.push(SavedSubjectRow{identity,ordinal,live,target_type:r.take(n)?.to_vec()});
        }
        if r.at!=bytes.len(){return Err(Failure::InvalidInput)}
        rows.sort_by_key(|r|r.ordinal);let candidate=Self{rows,next,..self.clone()};
        if candidate.encode()?!=bytes{return Err(Failure::SubjectCustodyInvalid)}Ok(candidate)
    }
}
