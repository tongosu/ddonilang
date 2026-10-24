//! Private SourceTransition bytes (WO22 exact R1.1). A decoded record is
//! evidence only: it cannot issue a run permit, reset FirstUse, or commit SQL.
use super::Failure;
const MAGIC: &[u8] = b"DDN-NATIVE-SOURCE-TRANSITION-V1\0";
const MAX_BYTES: usize = 12_698;
const LENGTHS: [usize; 17] = [32,8,8,1,32,0,32,8,32,1,0,32,0,32,32,32,8];

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(in crate::snapshot_v2_durable_transaction::native) enum Phase { Prepared, Ready, Consumed, Bound }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(in crate::snapshot_v2_durable_transaction::native) struct Record {
    pub nonce: [u8;32], pub generation: u64, pub ordinal: u64, pub phase: Phase,
    pub anchor_digest: [u8;32], pub anchor_revision: Vec<u8>,
    pub anchor_integrity: [u8;32], pub anchor_epoch: u64, pub anchor_record: [u8;32],
    pub parent_kind: u8, pub parent_revision: Vec<u8>, pub parent_integrity: [u8;32],
    pub source_revision: Vec<u8>, pub source_integrity: [u8;32], pub source_pesi: [u8;32],
    pub bound_digest: [u8;32], pub first_bound_epoch: u64,
}
impl Record {
    /// Record ordering only. This does not establish an authoring base or
    /// manufacture a run permit from decoded bytes.
    pub(super) fn check_successor(&self,previous:Option<&Self>)->Result<(),Failure>{
        self.validate()?;
        let Some(old)=previous else {
            return if self.ordinal==1&&self.phase==Phase::Prepared {Ok(())}else{Err(Failure::CustodyConflict)};
        };
        old.validate()?;
        if self.nonce!=old.nonce||self.generation!=old.generation{return Err(Failure::CustodyConflict);}
        if self==old{return Ok(());}
        let mut expected=old.clone();
        match (old.phase,self.phase){
            (Phase::Prepared,Phase::Ready)|(Phase::Ready,Phase::Consumed)=>expected.phase=self.phase,
            (Phase::Consumed,Phase::Bound)=>{expected.phase=Phase::Bound;expected.bound_digest=self.bound_digest;
                expected.first_bound_epoch=old.anchor_epoch.checked_add(1).ok_or(Failure::LimitExceeded)?;},
            (Phase::Ready,Phase::Prepared)=>{
                expected.ordinal=old.ordinal.checked_add(1).ok_or(Failure::LimitExceeded)?;expected.phase=Phase::Prepared;
                expected.parent_kind=0;expected.parent_revision=old.source_revision.clone();expected.parent_integrity=old.source_integrity;
                expected.source_revision=self.source_revision.clone();expected.source_integrity=self.source_integrity;expected.source_pesi=self.source_pesi;
            },
            (Phase::Bound,Phase::Prepared)=>{
                if old.ordinal.checked_add(1)!=Some(self.ordinal)||self.parent_kind!=1
                    ||self.parent_revision!=self.anchor_revision||self.parent_integrity!=self.anchor_integrity {
                    return Err(Failure::CustodyConflict);
                }
                // The actual mixed-history owner must prove the new anchor is
                // the current committed descendant, not simply this record.
                return Ok(());
            },
            _=>return Err(Failure::CustodyConflict),
        }
        if self!=&expected{return Err(Failure::CustodyConflict);}Ok(())
    }
    fn validate(&self)->Result<(),Failure> {
        if self.nonce==[0;32] || self.ordinal==0 || self.anchor_epoch==0 || self.parent_kind>1
            || [&self.anchor_revision,&self.parent_revision,&self.source_revision].iter().any(|v|v.is_empty()||v.len()>4096)
            || match self.phase {
                Phase::Bound=>self.anchor_epoch.checked_add(1)!=Some(self.first_bound_epoch),
                _=>self.bound_digest!=[0;32]||self.first_bound_epoch!=0,
            } {return Err(Failure::CorruptRecoveryRecord);}
        Ok(())
    }
    pub(super) fn encode(&self)->Result<Vec<u8>,Failure> {
        self.validate()?;
        let generation=self.generation.to_be_bytes();let ordinal=self.ordinal.to_be_bytes();
        let epoch=self.anchor_epoch.to_be_bytes();let bound=self.first_bound_epoch.to_be_bytes();
        let phase=[match self.phase{Phase::Prepared=>0,Phase::Ready=>1,Phase::Consumed=>2,Phase::Bound=>3}];
        let kind=[self.parent_kind];
        let fields:[&[u8];17]=[&self.nonce,&generation,&ordinal,&phase,&self.anchor_digest,&self.anchor_revision,
            &self.anchor_integrity,&epoch,&self.anchor_record,&kind,&self.parent_revision,&self.parent_integrity,
            &self.source_revision,&self.source_integrity,&self.source_pesi,&self.bound_digest,&bound];
        let length=fields.iter().try_fold(MAGIC.len()+3,|n,v|n.checked_add(5+v.len())).ok_or(Failure::LimitExceeded)?;
        if length>MAX_BYTES{return Err(Failure::LimitExceeded);}
        let mut bytes=Vec::new();bytes.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend(MAGIC);bytes.extend([0,1,17]);
        for (i,value) in fields.into_iter().enumerate(){bytes.push(i as u8+1);bytes.extend((value.len() as u32).to_be_bytes());bytes.extend(value);}
        Ok(bytes)
    }
    pub(super) fn decode(bytes:&[u8])->Result<Self,Failure> {
        if bytes.len()>MAX_BYTES || !bytes.starts_with(MAGIC)
            || bytes.get(MAGIC.len()..MAGIC.len()+3)!=Some(&[0,1,17]) {return Err(Failure::CorruptRecoveryRecord);}
        let mut cursor=MAGIC.len()+3;let mut fields:[&[u8];17]=[&[];17];
        // Inspect every tag, length, budget and EOF before allocating any field.
        for (i,fixed) in LENGTHS.into_iter().enumerate(){
            if bytes.get(cursor)!=Some(&(i as u8+1)){return Err(Failure::CorruptRecoveryRecord);}
            let header=bytes.get(cursor+1..cursor+5).ok_or(Failure::CorruptRecoveryRecord)?;
            let length=u32::from_be_bytes(header.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize;
            if (fixed!=0&&length!=fixed)||(fixed==0&&!(1..=4096).contains(&length)){return Err(Failure::CorruptRecoveryRecord);}
            cursor+=5;let end=cursor.checked_add(length).filter(|n|*n<=MAX_BYTES).ok_or(Failure::CorruptRecoveryRecord)?;
            fields[i]=bytes.get(cursor..end).ok_or(Failure::CorruptRecoveryRecord)?;cursor=end;
        }
        if cursor!=bytes.len(){return Err(Failure::CorruptRecoveryRecord);}
        let array=|i:usize|->Result<[u8;32],Failure>{fields[i].try_into().map_err(|_|Failure::CorruptRecoveryRecord)};
        let number=|i:usize|->Result<u64,Failure>{Ok(u64::from_be_bytes(fields[i].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?))};
        let value=Self{nonce:array(0)?,generation:number(1)?,ordinal:number(2)?,
            phase:match fields[3]{[0]=>Phase::Prepared,[1]=>Phase::Ready,[2]=>Phase::Consumed,[3]=>Phase::Bound,_=>return Err(Failure::CorruptRecoveryRecord)},
            anchor_digest:array(4)?,anchor_revision:fields[5].to_vec(),anchor_integrity:array(6)?,anchor_epoch:number(7)?,
            anchor_record:array(8)?,parent_kind:fields[9][0],parent_revision:fields[10].to_vec(),parent_integrity:array(11)?,
            source_revision:fields[12].to_vec(),source_integrity:array(13)?,source_pesi:array(14)?,bound_digest:array(15)?,first_bound_epoch:number(16)?};
        value.validate()?;Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record()->Record{Record{nonce:[1;32],generation:0,ordinal:1,phase:Phase::Prepared,
        anchor_digest:[2;32],anchor_revision:vec![3],anchor_integrity:[4;32],anchor_epoch:5,anchor_record:[6;32],
        parent_kind:1,parent_revision:vec![7],parent_integrity:[8;32],source_revision:vec![9],
        source_integrity:[10;32],source_pesi:[11;32],bound_digest:[0;32],first_bound_epoch:0}}
    #[test]
    fn exact_transition_codec_phase_and_bound_epoch(){
        let mut r=record();
        for phase in [Phase::Prepared,Phase::Ready,Phase::Consumed,Phase::Bound]{
            r.phase=phase;if phase==Phase::Bound{r.bound_digest=[12;32];r.first_bound_epoch=6;}
            let bytes=r.encode().unwrap();assert_eq!(Record::decode(&bytes).unwrap(),r);
        }
        r.first_bound_epoch=7;assert!(r.encode().is_err());r.anchor_epoch=u64::MAX;r.first_bound_epoch=0;assert!(r.encode().is_err());
    }
    #[test]
    fn exact_transition_codec_limits_and_corrupt_framing(){
        let mut r=record();r.anchor_revision=vec![1;4096];r.parent_revision=vec![2;4096];r.source_revision=vec![3;4096];
        let bytes=r.encode().unwrap();assert_eq!(bytes.len(),MAX_BYTES);assert_eq!(Record::decode(&bytes).unwrap(),r);
        let mut bad=bytes.clone();bad.push(0);assert!(Record::decode(&bad).is_err());
        let mut bad=bytes.clone();bad[MAGIC.len()+3]=2;assert!(Record::decode(&bad).is_err());
        let mut bad=bytes.clone();bad[MAGIC.len()+4..MAGIC.len()+8].copy_from_slice(&u32::MAX.to_be_bytes());assert!(Record::decode(&bad).is_err());
        assert!(Record::decode(&bytes[..bytes.len()-1]).is_err());r.source_revision.clear();assert!(r.encode().is_err());
        let mut r=record();r.first_bound_epoch=1;assert!(r.encode().is_err());r.first_bound_epoch=0;r.nonce=[0;32];assert!(r.encode().is_err());
    }
    #[test]
    fn transition_record_order_is_not_restart_permission(){
        let prepared=record();prepared.check_successor(None).unwrap();let mut ready=prepared.clone();ready.phase=Phase::Ready;ready.check_successor(Some(&prepared)).unwrap();
        let mut consumed=ready.clone();consumed.phase=Phase::Consumed;consumed.check_successor(Some(&ready)).unwrap();
        let mut bound=consumed.clone();bound.phase=Phase::Bound;bound.bound_digest=[12;32];bound.first_bound_epoch=6;bound.check_successor(Some(&consumed)).unwrap();
        assert!(ready.check_successor(Some(&consumed)).is_err());assert!(consumed.check_successor(None).is_err());
        let mut edit=ready.clone();edit.phase=Phase::Prepared;edit.ordinal=2;edit.parent_kind=0;edit.parent_revision=ready.source_revision.clone();edit.parent_integrity=ready.source_integrity;edit.source_revision=vec![20];edit.check_successor(Some(&ready)).unwrap();
        edit.anchor_epoch+=1;assert!(edit.check_successor(Some(&ready)).is_err());
    }
    #[test]
    fn approved_r11_codec_vectors_only_not_dpapi_or_run(){
        let v:serde_json::Value=serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/data/wo22_source_transition_r11_vectors.json"))).unwrap();
        for case in v["codec_positives"].as_array().unwrap(){
            let bytes=hex::decode(case["hex"].as_str().unwrap()).unwrap();
            let decoded=Record::decode(&bytes).unwrap_or_else(|e|panic!("{}: {e:?}",case["id"]));
            assert_eq!(decoded.encode().unwrap(),bytes,"{}",case["id"]);
        }
        for case in v["codec_negatives"].as_array().unwrap(){
            let bytes=hex::decode(case["hex"].as_str().unwrap()).unwrap();assert!(Record::decode(&bytes).is_err(),"{}",case["id"]);
        }
    }
}
