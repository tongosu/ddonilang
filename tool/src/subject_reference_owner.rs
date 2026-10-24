//! RD-665/RD-668: one shared saved-world identity custody verifier.
//! Input PESI is already admitted by its issuer; this owner never invents it.
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use sha2::{Digest,Sha256};
mod temporary;
pub(crate) use temporary::TemporarySubjectOwnerV1;

#[cfg(test)]
mod tests{
    use super::*;
    const PESI:&str="9497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad4";
    // Exact SWC2-P02 bytes, R2.1 authoritative vector SHA02786002...10969.
    const LIVE:&str="44444e2d53415645442d574f524c442d4944454e544954592d435553544f44592d5631000001000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f56ef58c78d42e6499fa62157839db76abab11bdf117a1f7e4bf9a6e0917c01b2000000209497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad400000000000000010000000000000001cb6695c2db6786c3d0843d02e5a4cf9172a485206a68aa5a53c351c524a9db6a0000000000000000000000000000000001000000021100";
    fn pesi()->[u8;32]{hex::decode(PESI).unwrap().try_into().unwrap()}
    fn load(raw:&[u8])->Result<SavedWorldIdentityCustodyV1,Failure>{
        SavedWorldIdentityCustodyV1::load_recorded(raw,raw.len() as u64,Sha256::digest(raw).into(),&pesi())
    }
    #[test]
    fn beta1_subject_exact_r2_vectors_keep_pesi_world_and_tombstone(){
        let raw=hex::decode(LIVE).unwrap();assert_eq!(raw.len(),209);
        assert_eq!(hex::encode(Sha256::digest(&raw)),"dd2b4c2916ff4aad794d7b562c3f7ef3b07fd6e406e9e8b68c74ba591bc26a59");
        let custody=load(&raw).unwrap();
        assert_eq!(custody.encode_recorded().unwrap(), raw);
        assert_eq!(custody.pesi,pesi());assert_eq!(custody.next_subject_ordinal,1);
        assert_eq!(hex::encode(custody.world),"56ef58c78d42e6499fa62157839db76abab11bdf117a1f7e4bf9a6e0917c01b2");
        assert_eq!(custody.rows[0].ordinal,0);assert!(custody.rows[0].live);
        assert_eq!(custody.rows[0].target_type,[0x11,0]);
        assert_eq!(hex::encode(custody.rows[0].identity),"cb6695c2db6786c3d0843d02e5a4cf9172a485206a68aa5a53c351c524a9db6a");
        let mut tombstone=raw.clone();tombstone[202]=2;
        assert_eq!(hex::encode(Sha256::digest(&tombstone)),"360b0b4a9d2d4f7f840d65862c25801232a90e1584b260e87d206859fc4694ea");
        let dead=load(&tombstone).unwrap();assert!(!dead.rows[0].live);
        assert_eq!(dead.rows[0].identity,custody.rows[0].identity);
        let mut empty=raw[..154].to_vec();empty[138..154].fill(0);
        assert_eq!(hex::encode(Sha256::digest(&empty)),"a3eb29e94945d762f1f5d02ffc76157a63575ec442d15a8510fc9e27cdbe1861");
        assert!(load(&empty).unwrap().rows.is_empty());
    }
    #[test]
    fn beta1_subject_failures_never_repair_pesi_generation_or_identity(){
        let raw=hex::decode(LIVE).unwrap();
        for n in 0..raw.len(){assert!(load(&raw[..n]).is_err(),"truncation {n}");}
        for offset in [34,36,68,105,145,154,193,201,202]{
            let mut bad=raw.clone();bad[offset]^=0xff;
            assert!(load(&bad).is_err(),"modified {offset} with recomputed integrity");
        }
        let mut old_short=raw.clone();old_short[102..106].copy_from_slice(&2u32.to_be_bytes());
        assert!(load(&old_short).is_err());
        let wrong=[0;32];
        assert!(matches!(SavedWorldIdentityCustodyV1::load_recorded(&raw,209,Sha256::digest(&raw).into(),&wrong),Err(Failure::ForeignPesi)));
        assert!(SavedWorldIdentityCustodyV1::load_recorded(&raw,208,Sha256::digest(&raw).into(),&pesi()).is_err());
        assert!(SavedWorldIdentityCustodyV1::load_recorded(&raw,209,[0;32],&pesi()).is_err());
        let mut suffix=raw.clone();suffix.push(0);assert!(load(&suffix).is_err());
    }
    #[test]
    fn beta1_subject_named_limits_are_checked_before_row_materialization(){
        let raw=hex::decode(LIVE).unwrap();
        let mut too_many=raw.clone();too_many[146..154].copy_from_slice(&65537u64.to_be_bytes());
        assert!(matches!(load(&too_many),Err(Failure::LimitExceeded)));
        let mut huge_type=raw.clone();huge_type[203..207].copy_from_slice(&4097u32.to_be_bytes());
        assert!(matches!(load(&huge_type),Err(Failure::LimitExceeded)));
        let mut exact_type=raw[..203].to_vec();exact_type.extend(4096u32.to_be_bytes());exact_type.extend(vec![0x11;4096]);
        assert_eq!(load(&exact_type).unwrap().rows[0].target_type.len(),4096);
        assert!(matches!(load(&vec![0;1_000_001]),Err(Failure::LimitExceeded)));
    }

    #[test]
    fn regular_native_subject_owner_keeps_abort_tombstones_and_cold_next_ordinal() {
        let mut owner = LiveSubjectOwnerV1::fresh(pesi()).unwrap();
        let initial = owner.custody().encode_recorded().unwrap();
        let aborted = owner.prepare_allocation(&[0x11, 0]).unwrap();
        drop(aborted);
        assert_eq!(owner.custody().encode_recorded().unwrap(), initial);
        let pending = owner.prepare_allocation(&[0x11, 0]).unwrap();
        let stale = owner.prepare_allocation(&[0x11, 0]).unwrap();
        let first = owner.commit_allocation(pending).unwrap();
        let committed = owner.custody().encode_recorded().unwrap();
        assert!(owner.commit_allocation(stale).is_err());
        assert_eq!(owner.custody().encode_recorded().unwrap(), committed);
        owner.retire(&first).unwrap();
        let saved = owner.custody().encode_recorded().unwrap();
        let recorded = SavedWorldIdentityCustodyV1::load_recorded(
            &saved, saved.len() as u64, Sha256::digest(&saved).into(), &pesi()).unwrap();
        let mut restored = LiveSubjectOwnerV1::from_recorded_candidate(recorded).unwrap();
        assert_eq!(restored.custody().encode_recorded().unwrap(), saved);
        assert!(restored.retire(&first).is_err());
        let next = restored.prepare_allocation(&[0x11, 0]).unwrap();
        let second = restored.commit_allocation(next).unwrap();
        assert_eq!(first.owner_space_identity, second.owner_space_identity);
        assert_ne!(first.target_identity, second.target_identity);
        assert_eq!(restored.custody().next_subject_ordinal, 2);
        assert!(!restored.custody().rows[0].live);
        assert_eq!(restored.custody().rows[1].ordinal, 1);
        SubjectReferenceContextV1::new(restored.custody()).unwrap().verify(&second).unwrap();
        assert!(SubjectReferenceContextV1::new(restored.custody()).unwrap().verify(&first).is_err());
    }

    #[test]
    fn regular_native_owner_rejects_foreign_pending_and_invalid_type_without_advance() {
        let mut owner = LiveSubjectOwnerV1::fresh(pesi()).unwrap();
        let other = LiveSubjectOwnerV1::fresh(pesi()).unwrap();
        let before = owner.custody().encode_recorded().unwrap();
        assert!(owner.prepare_allocation(&[]).is_err());
        assert!(owner.prepare_allocation(&vec![1; 4097]).is_err());
        assert!(owner.commit_allocation(other.prepare_allocation(&[1]).unwrap()).is_err());
        assert_eq!(owner.custody().encode_recorded().unwrap(), before);
        // Exercise only the semantic exhaustion guard, not a fabricated
        // admitted snapshot of u64::MAX rows (which exceeds saved bounds).
        owner.custody.next_subject_ordinal = u64::MAX;
        assert!(matches!(owner.prepare_allocation(&[1]), Err(Failure::LimitExceeded)));
        assert_eq!(owner.custody.next_subject_ordinal, u64::MAX);
    }
}

#[derive(Debug,Clone)]
pub(crate) struct SavedSubjectRow {
    pub(crate) identity:[u8;32],
    pub(crate) ordinal:u64,
    pub(crate) live:bool,
    pub(crate) target_type:Vec<u8>,
}
#[derive(Debug,Clone)]
pub(crate) struct SavedWorldIdentityCustodyV1 {
    pub(crate) nonce:[u8;32],
    pub(crate) world:[u8;32],
    pub(crate) pesi:[u8;32],
    pub(crate) next_subject_ordinal:u64,
    pub(crate) rows:Vec<SavedSubjectRow>,
}

/// RD-665 regular world/Subject issuer, not a save-time repair mechanism.
/// The execution owner must stage this alongside its world and publish both
/// only after the enclosing operation succeeds. No adapter can supply a nonce.
#[derive(Clone)]
pub(crate) struct LiveSubjectOwnerV1 {
    custody: SavedWorldIdentityCustodyV1,
}

pub(crate) struct PendingSubjectAllocationV1 {
    world: [u8; 32],
    pesi: [u8; 32],
    row: SavedSubjectRow,
}

impl LiveSubjectOwnerV1 {
    pub(crate) fn fresh(admitted_pesi: [u8; 32]) -> Result<Self, Failure> {
        let mut nonce = [0; 32];
        getrandom::fill(&mut nonce).map_err(|_| Failure::SubjectCustodyInvalid)?;
        let custody = SavedWorldIdentityCustodyV1 {
            nonce, world: recorded_world_identity(&nonce)?, pesi: admitted_pesi,
            next_subject_ordinal: 0, rows: Vec::new(),
        };
        Ok(Self { custody })
    }

    /// This is a staging owner. Caller still owes complete FRAME2 source,
    /// provenance, replay and durability admission before live publication.
    pub(crate) fn from_recorded_candidate(custody: SavedWorldIdentityCustodyV1) -> Result<Self, Failure> {
        custody.encode_recorded()?;
        Ok(Self { custody })
    }

    pub(crate) fn prepare_allocation(&self, target_type: &[u8]) -> Result<PendingSubjectAllocationV1, Failure> {
        if target_type.is_empty() || target_type.len() > 4096 {
            return Err(Failure::SubjectCustodyInvalid);
        }
        // Snapshot row/byte limits govern saving, not the semantic ordinal
        // range of regular Subject creation. Never reset an exhausted counter.
        if self.custody.next_subject_ordinal == u64::MAX {
            return Err(Failure::LimitExceeded);
        }
        let ordinal = self.custody.next_subject_ordinal;
        Ok(PendingSubjectAllocationV1 {
            world: self.custody.world, pesi: self.custody.pesi,
            row: SavedSubjectRow { identity: recorded_subject_identity(&self.custody.world, ordinal),
                ordinal, live: true, target_type: target_type.to_vec() },
        })
    }

    pub(crate) fn commit_allocation(&mut self, pending: PendingSubjectAllocationV1)
        -> Result<ddonirang_core::platform::IdentityReferenceValueV1, Failure> {
        if pending.world != self.custody.world || pending.pesi != self.custody.pesi
            || pending.row.ordinal != self.custody.next_subject_ordinal {
            return Err(Failure::SubjectCustodyInvalid);
        }
        if self.custody.next_subject_ordinal == u64::MAX { return Err(Failure::LimitExceeded); }
        self.custody.rows.try_reserve(1).map_err(|_| Failure::LimitExceeded)?;
        let reference = ddonirang_core::platform::IdentityReferenceValueV1 {
            profile_version: 1, target_type_identity: pending.row.target_type.clone(),
            owner_space_identity: self.custody.world.to_vec(),
            target_identity: pending.row.identity.to_vec(), generation: 0,
        };
        self.custody.rows.push(pending.row);
        self.custody.next_subject_ordinal += 1;
        Ok(reference)
    }

    pub(crate) fn retire(&mut self, reference: &ddonirang_core::platform::IdentityReferenceValueV1) -> Result<(), Failure> {
        SubjectReferenceContextV1::new(&self.custody)?.verify(reference)?;
        let row = self.custody.rows.iter_mut().find(|row| row.identity.as_slice() == reference.target_identity)
            .ok_or(Failure::SubjectCustodyInvalid)?;
        row.live = false;
        Ok(())
    }

    pub(crate) fn custody(&self) -> &SavedWorldIdentityCustodyV1 { &self.custody }
}

/// One lookup index for an admitted saved snapshot, never an ambient registry.
/// Its lifetime ties all nested reference admission to the same PESI/world.
pub(crate) struct SubjectReferenceContextV1<'a> {
    world:&'a [u8;32],
    rows:std::collections::BTreeMap<[u8;32],&'a SavedSubjectRow>,
}
impl<'a> SubjectReferenceContextV1<'a> {
    pub(crate) fn verify_handle(&self,value:&ddonirang_core::nominal::SubjectHandleV1)->Result<(),Failure>{
        let row=self.rows.get(&value.subject_identity).ok_or(Failure::SubjectCustodyInvalid)?;
        if value.world_identity!=*self.world||!row.live||row.target_type.as_slice()!=value.name_type_identity {
            return Err(Failure::SubjectCustodyInvalid);
        }
        Ok(())
    }
    pub(crate) fn new(custody:&'a SavedWorldIdentityCustodyV1)->Result<Self,Failure>{
        Self::from_lifecycle_parts(&custody.world,&custody.rows)
    }
    fn from_lifecycle_parts(world:&'a [u8;32],lifecycle:&'a [SavedSubjectRow])->Result<Self,Failure>{
        if lifecycle.len()>65_536 {return Err(Failure::LimitExceeded);}
        let mut rows=std::collections::BTreeMap::new();
        for row in lifecycle {
            if rows.insert(row.identity,row).is_some(){return Err(Failure::SubjectCustodyInvalid);}
        }
        Ok(Self{world,rows})
    }
    pub(crate) fn verify(&self,value:&ddonirang_core::platform::IdentityReferenceValueV1)->Result<(),Failure>{
        if value.profile_version!=1||value.generation!=0||value.target_type_identity.is_empty()
            ||value.target_type_identity.len()>4096||value.owner_space_identity.as_slice()!=self.world
            ||value.target_identity.len()!=32 {
            return Err(Failure::SubjectCustodyInvalid);
        }
        let identity:&[u8;32]=value.target_identity.as_slice().try_into().map_err(|_|Failure::SubjectCustodyInvalid)?;
        let row=self.rows.get(identity).ok_or(Failure::SubjectCustodyInvalid)?;
        if !row.live||value.target_type_identity!=row.target_type {return Err(Failure::SubjectCustodyInvalid);}
        Ok(())
    }
}
fn recorded_world_identity(nonce: &[u8; 32]) -> Result<[u8; 32], Failure> {
    if nonce.iter().all(|byte| *byte == 0) {
        return Err(Failure::SubjectCustodyInvalid);
    }
    let mut hash = Sha256::new();
    hash.update(b"DDN-WORLD-SPACE-IDENTITY-V1\0");
    hash.update(1u16.to_be_bytes());
    hash.update([1, 1]);
    hash.update(32u32.to_be_bytes());
    hash.update(nonce);
    Ok(hash.finalize().into())
}

fn recorded_subject_identity(world: &[u8; 32], ordinal: u64) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"DDN-SUBJECT-IDENTITY-V1\0");
    hash.update(1u16.to_be_bytes());
    hash.update([2, 1]);
    hash.update(32u32.to_be_bytes());
    hash.update(world);
    hash.update([2]);
    hash.update(8u32.to_be_bytes());
    hash.update(ordinal.to_be_bytes());
    hash.finalize().into()
}


struct Cursor<'a>{bytes:&'a[u8],at:usize}
impl<'a> Cursor<'a>{
    fn take(&mut self,n:usize)->Result<&'a[u8],Failure>{
        let end=self.at.checked_add(n).ok_or(Failure::LimitExceeded)?;
        let bytes=self.bytes.get(self.at..end).ok_or(Failure::SubjectCustodyInvalid)?;
        self.at=end;Ok(bytes)
    }
    fn fixed<const N:usize>(&mut self)->Result<[u8;N],Failure>{
        self.take(N)?.try_into().map_err(|_|Failure::SubjectCustodyInvalid)
    }
    fn u64(&mut self)->Result<u64,Failure>{Ok(u64::from_be_bytes(self.fixed()?))}
    fn row(&mut self)->Result<([u8;32],u64,bool,&'a[u8]),Failure>{
        let identity=self.fixed()?;
        let ordinal=self.u64()?;let generation=self.u64()?;let lifecycle=self.take(1)?[0];
        let length=u32::from_be_bytes(self.fixed()?) as usize;
        if length>4096{return Err(Failure::LimitExceeded);}
        if length==0||generation!=0||!matches!(lifecycle,1|2){return Err(Failure::SubjectCustodyInvalid);}
        Ok((identity,ordinal,lifecycle==1,self.take(length)?))
    }
    fn done(&self)->Result<(),Failure>{
        if self.at==self.bytes.len(){Ok(())}else{Err(Failure::SubjectCustodyInvalid)}
    }
}
impl SavedWorldIdentityCustodyV1 {
    pub(crate) fn recorded_byte_length(&self) -> Result<usize, Failure> {
        const DOMAIN: &[u8] = b"DDN-SAVED-WORLD-IDENTITY-CUSTODY-V1\0";
        if self.rows.len() > 65_536 { return Err(Failure::LimitExceeded); }
        if recorded_world_identity(&self.nonce)? != self.world
            || self.next_subject_ordinal != self.rows.len() as u64 { return Err(Failure::SubjectCustodyInvalid); }
        let mut length = DOMAIN.len() + 2 + 32 + 32 + 4 + 32 + 8 + 8;
        for (ordinal, row) in self.rows.iter().enumerate() {
            if row.target_type.is_empty() || row.target_type.len() > 4096 { return Err(Failure::SubjectCustodyInvalid); }
            if row.ordinal != ordinal as u64 || row.identity != recorded_subject_identity(&self.world, row.ordinal) {
                return Err(Failure::SubjectCustodyInvalid);
            }
            length = length.checked_add(53 + row.target_type.len()).ok_or(Failure::LimitExceeded)?;
            if length > 1_000_000 { return Err(Failure::LimitExceeded); }
        }
        Ok(length)
    }

    pub(crate) fn encode_recorded(&self) -> Result<Vec<u8>, Failure> {
        const DOMAIN: &[u8] = b"DDN-SAVED-WORLD-IDENTITY-CUSTODY-V1\0";
        let length = self.recorded_byte_length()?;
        let mut raw = Vec::new();
        raw.try_reserve_exact(length).map_err(|_| Failure::LimitExceeded)?;
        raw.extend(DOMAIN); raw.extend(1u16.to_be_bytes());
        raw.extend(self.nonce); raw.extend(self.world); raw.extend(32u32.to_be_bytes()); raw.extend(self.pesi);
        raw.extend(self.next_subject_ordinal.to_be_bytes()); raw.extend((self.rows.len() as u64).to_be_bytes());
        for row in &self.rows {
            raw.extend(row.identity); raw.extend(row.ordinal.to_be_bytes()); raw.extend(0u64.to_be_bytes());
            raw.push(if row.live { 1 } else { 2 });
            raw.extend((row.target_type.len() as u32).to_be_bytes()); raw.extend(&row.target_type);
        }
        Ok(raw)
    }

    pub(crate) fn load_recorded(raw:&[u8],byte_length:u64,sha256:[u8;32],admitted_pesi:&[u8;32])->Result<Self,Failure>{
        if raw.len()>1_000_000||byte_length>1_000_000{return Err(Failure::LimitExceeded);}
        if raw.len() as u64!=byte_length||<[u8;32]>::from(Sha256::digest(raw))!=sha256{return Err(Failure::SubjectCustodyInvalid);}
        let mut cursor=Cursor{bytes:raw,at:0};
        const DOMAIN:&[u8]=b"DDN-SAVED-WORLD-IDENTITY-CUSTODY-V1\0";
        if cursor.take(DOMAIN.len())?!=DOMAIN||cursor.take(2)?!=[0,1]{return Err(Failure::SubjectCustodyInvalid);}
        let nonce=cursor.fixed()?;let world=cursor.fixed()?;
        if recorded_world_identity(&nonce)?!=world{return Err(Failure::SubjectCustodyInvalid);}
        if cursor.take(4)?!=32u32.to_be_bytes(){return Err(Failure::ForeignPesi);}
        let pesi=cursor.fixed()?;
        if &pesi!=admitted_pesi{return Err(Failure::ForeignPesi);}
        let next_subject_ordinal=cursor.u64()?;let count=cursor.u64()?;
        if count>65536{return Err(Failure::LimitExceeded);}
        if count!=next_subject_ordinal||count>(raw.len().saturating_sub(cursor.at)/54) as u64{return Err(Failure::SubjectCustodyInvalid);}
        let rows_at=cursor.at;
        // All row bounds and identities are checked before vector/type allocation.
        let mut tombstones=0u64;
        for ordinal in 0..count {
            let (identity,stored_ordinal,live,_)=cursor.row()?;
            if stored_ordinal!=ordinal||identity!=recorded_subject_identity(&world,ordinal){return Err(Failure::SubjectCustodyInvalid);}
            if !live{tombstones+=1;if tombstones>65536{return Err(Failure::LimitExceeded);}}
        }
        cursor.done()?;
        cursor.at=rows_at;
        let mut rows=Vec::new();rows.try_reserve_exact(count as usize).map_err(|_|Failure::LimitExceeded)?;
        for _ in 0..count{
            let(identity,ordinal,live,target_type)=cursor.row()?;
            rows.push(SavedSubjectRow{identity,ordinal,live,target_type:target_type.to_vec()});
        }
        Ok(Self{nonce,world,pesi,next_subject_ordinal,rows})
    }
}
