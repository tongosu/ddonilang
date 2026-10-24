// R2 evidence lives in its own protected value/entropy. Never alias LC/Attempt.
// Included only in the existing Windows grant owner; no public raw issuer.
const FINISH_WITNESS_DOMAIN:&[u8]=b"DDN-WO25-FINISH-RECOVERY-WITNESS-V1\0";
#[cfg(test)]
std::thread_local!{static FAIL_NEXT_FINISH_READY_SET:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
#[cfg(test)]
pub(super) fn test_arm_finish_ready_set_failure(){FAIL_NEXT_FINISH_READY_SET.with(|fault|fault.set(true));}

// Observation only: no plaintext, grant, permit or registry write is returned.
#[cfg(test)]
pub(super) fn test_finish_lifecycle_cipher_digest(nonce:&[u8;32],first:&[u8;32])->Result<[u8;32],Failure>{
    use sha2::{Digest,Sha256};
    let key=WindowsRootGrantStoreV1::key(Namespace::Grant,nonce,false,false)?;
    let name=utf16(&format!("WO23ReplayState-{}",hex::encode(first)));
    let mut kind=0;let mut length=0;
    // SAFETY: live key and bounded buffers, using the existing registry API.
    let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
    if status!=0||kind!=3||length==0||length as usize>MAX_CIPHERTEXT_BYTES{return Err(Failure::CorruptRecoveryRecord);}
    let expected=length;let mut bytes=vec![0;length as usize];
    let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,bytes.as_mut_ptr(),&mut length)};
    if status!=0||kind!=3||length!=expected{return Err(Failure::RereadFailed);}
    Ok(Sha256::digest(&bytes).into())
}

fn finish_witness_locator(header:&crate::replay_evidence::finish_recovery::RecordHeaderV1<'_>)
    ->Result<(Vec<u16>,Vec<u8>),Failure>{
    use crate::replay_evidence::finish_recovery::RecordKindV1;
    if header.kind()!=RecordKindV1::Intent{return Err(Failure::InvalidInput);}
    let field=|tag|header.field(tag).ok_or(Failure::InvalidInput);
    let nonce=field(2)?.try_into().map_err(|_|Failure::InvalidInput)?;
    let first=field(5)?.try_into().map_err(|_|Failure::InvalidInput)?;
    let epoch=u64::from_be_bytes(field(6)?.try_into().map_err(|_|Failure::InvalidInput)?);
    let ordinal=u64::from_be_bytes(field(7)?.try_into().map_err(|_|Failure::InvalidInput)?);
    Ok(finish_witness_location(nonce,first,epoch,ordinal))
}

fn finish_witness_location(nonce:&[u8;32],first:&[u8;32],epoch:u64,ordinal:u64)->(Vec<u16>,Vec<u8>){
    let name=utf16(&format!("WO25FinishRecovery-{}-{epoch:016x}-{ordinal:016x}",hex::encode(first)));
    let mut entropy=FINISH_WITNESS_DOMAIN.to_vec();
    entropy.extend_from_slice(nonce);entropy.extend_from_slice(first);
    entropy.extend_from_slice(&epoch.to_be_bytes());entropy.extend_from_slice(&ordinal.to_be_bytes());
    (name,entropy)
}

impl ReplayArchiveRootScopeV1 {
    pub(crate) fn write_finish_pending(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,witness:&DurableFinishIntentWitnessV1)
        ->Result<(),super::super::CapturedReplayMetadataFailureV1>{
        let before=|cause|finish_lifecycle_failure(intent,1,cause,false,true);
        self.verify_initial_finish_witness(adapter,intent,witness).map_err(before)?;
        let pending=intent.pending().ok_or_else(||before(Failure::InvalidInput))?;
        let mut current=self.read_replay_lifecycle(pending.first_tx()).map_err(before)?;
        let same=current.as_deref()==pending.previous_bytes();
        if let Some(raw)=&mut current{raw.fill(0);}
        if !same{return Err(before(Failure::ConflictingRetry));}
        self.write_finish_lifecycle_bytes(adapter,intent,pending.canonical_bytes(),false)
    }

    pub(crate) fn write_finish_ready(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,
        ready:&crate::replay_evidence::ReadyReplayLifecycleV1)
        ->Result<ProtectedReplayReadyV1,super::super::CapturedReplayMetadataFailureV1>{
        let before=|cause|finish_lifecycle_failure(intent,11,cause,true,false);
        self.verify_current(adapter).map_err(before)?;
        let mut pending=self.read_replay_lifecycle(ready.first_tx()).map_err(before)?
            .ok_or_else(||before(Failure::CustodyConflict))?;
        let same=pending==ready.expected_pending();pending.fill(0);
        if !same || ready.first_tx()!=&intent.row().first_tx{return Err(before(Failure::CustodyConflict));}
        let mut bytes=Vec::new();bytes.try_reserve_exact(ready.canonical_bytes().len()).map_err(|_|before(Failure::LimitExceeded))?;
        bytes.extend_from_slice(ready.canonical_bytes());
        self.write_finish_lifecycle_bytes(adapter,intent,ready.canonical_bytes(),true)?;
        let scope=ReplayArchiveRootScopeV1{record:self.record.clone(),pinned:self.pinned.clone(),lease:self.lease.clone()};
        Ok(ProtectedReplayReadyV1{scope,bytes,first_tx:*ready.first_tx()})
    }

    fn verify_initial_finish_witness(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,witness:&DurableFinishIntentWitnessV1)
        ->Result<(),Failure>{
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,witness::WitnessCandidateV1};
        adapter.verify_finish_recovery_intent(self,intent)?;
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Failure::InvalidInput)?;
        let candidate=WitnessCandidateV1::parse(&witness.plain).map_err(|_|Failure::InvalidInput)?;
        candidate.check_intent(&header,intent.record_digest()).map_err(|_|Failure::CustodyConflict)?;
        if candidate.stage()!=0 || witness.intent_digest!=*intent.record_digest(){return Err(Failure::CustodyConflict);}
        let mut actual=self.read_finish_witness(&header)?.ok_or(Failure::CustodyConflict)?;
        let same=actual==witness.plain;actual.fill(0);
        if !same{return Err(Failure::CustodyConflict);}
        self.verify_current(adapter)
    }

    fn write_finish_lifecycle_bytes(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,plain:&[u8],ready:bool)
        ->Result<(),super::super::CapturedReplayMetadataFailureV1>{
        use crate::replay_evidence::finish_recovery::RecordHeaderV1;
        let first_stage=if ready{11}else{1};
        let before=|cause|finish_lifecycle_failure(intent,first_stage,cause,ready,!ready);
        let uncertain=|stage,cause|finish_lifecycle_failure(intent,stage,cause,ready,false);
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|before(Failure::InvalidInput))?;
        if header.field(if ready{11}else{10})!=Some(plain){return Err(before(Failure::CustodyConflict));}
        self.verify_current(adapter).map_err(before)?;
        let first=&intent.row().first_tx;let entropy=self.replay_entropy(first);
        let mut inner=replay_domain_transform(plain,true,false,&entropy).map_err(before)?;
        let protected=replay_domain_transform(&inner,true,true,&entropy);inner.fill(0);
        let cipher=protected.map_err(before)?;
        if cipher.is_empty()||cipher.len()>MAX_CIPHERTEXT_BYTES{return Err(before(Failure::LimitExceeded));}
        let key=WindowsRootGrantStoreV1::key(Namespace::Grant,&self.record.nonce,true,false).map_err(before)?;
        let name=utf16(&format!("WO23ReplayState-{}",hex::encode(first)));
        // SAFETY: existing grant key and bounded live buffers. Stage is captured
        // here, not inferred later from a message or a numeric failure code.
        #[cfg(test)]
        if ready && FAIL_NEXT_FINISH_READY_SET.with(|fault|fault.replace(false)){
            return Err(uncertain(first_stage,Failure::WriteFailed));
        }
        if unsafe{RegSetValueExW(key.0,name.as_ptr(),0,3,cipher.as_ptr(),cipher.len() as u32)}!=0{
            return Err(uncertain(first_stage,Failure::WriteFailed));
        }
        let flush_stage=if ready{12}else{2};
        if unsafe{RegFlushKey(key.0)}!=0{return Err(uncertain(flush_stage,Failure::SyncFailed));}
        drop(key);
        let read_stage=if ready{13}else{3};
        let mut returned=self.read_replay_lifecycle(first).map_err(|cause|uncertain(read_stage,cause))?
            .ok_or_else(||uncertain(read_stage,Failure::RereadFailed))?;
        let same=returned==plain;returned.fill(0);
        if !same{return Err(uncertain(read_stage,Failure::RereadMismatch));}
        self.verify_current(adapter).map_err(|cause|uncertain(read_stage,cause))
    }

    /// The actual immutable producer is required. SQL candidates/host booleans
    /// cannot enter this API. It does not modify Pending or publish success.
    pub(crate) fn persist_finish_intent_witness(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1)
        ->Result<DurableFinishIntentWitnessV1,ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,witness};
        use sha2::{Digest,Sha256};
        adapter.verify_finish_recovery_intent(self,intent).map_err(Before)?;
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Before(Failure::InvalidInput))?;
        let field=|tag|header.field(tag).ok_or(Before(Failure::InvalidInput));
        let first:[u8;32]=field(5)?.try_into().map_err(|_|Before(Failure::InvalidInput))?;
        let pending_digest:[u8;32]=Sha256::digest(field(10)?).into();
        let zero=[0;32];let stage=[0];
        let plain=witness::encode(&[field(2)?,field(3)?,field(4)?,field(5)?,field(6)?,field(7)?,
            field(8)?,intent.record_digest(),&zero,&stage,&zero,&pending_digest])
            .map_err(|_|Before(Failure::InvalidInput))?;
        let mut current=self.read_replay_lifecycle(&first).map_err(Before)?;
        let matches=current.as_deref()==Some(field(9)?);
        if let Some(raw)=&mut current{raw.fill(0);}
        if !matches{return Err(Before(Failure::CustodyConflict));}
        self.write_initial_finish_witness(&header,&plain)?;
        // A successful DPAPI read alone never replaces current grant/lease.
        self.verify_current(adapter).map_err(Uncertain)?;
        adapter.verify_finish_recovery_intent(self,intent).map_err(Uncertain)?;
        Ok(DurableFinishIntentWitnessV1{plain,intent_digest:*intent.record_digest()})
    }

    fn read_finish_witness(&self,header:&crate::replay_evidence::finish_recovery::RecordHeaderV1<'_>)
        ->Result<Option<Vec<u8>>,Failure>{
        let (name,entropy)=finish_witness_locator(header)?;
        let key=WindowsRootGrantStoreV1::key(Namespace::Grant,&self.record.nonce,false,false)?;
        let mut kind=0;let mut length=0;
        // SAFETY: live key and initialized outputs; bounded allocation below.
        let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
        if status==2{return Ok(None);}
        if status!=0 || kind!=3 || length==0 || length as usize>MAX_CIPHERTEXT_BYTES{
            return Err(Failure::CorruptRecoveryRecord);
        }
        let expected=length;let mut cipher=Vec::new();
        cipher.try_reserve_exact(length as usize).map_err(|_|Failure::LimitExceeded)?;
        cipher.resize(length as usize,0);
        // SAFETY: allocated bounded buffer; reject a changed type/size.
        let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,cipher.as_mut_ptr(),&mut length)};
        if status!=0 || kind!=3 || length!=expected{return Err(Failure::RereadFailed);}
        drop(key);
        let mut inner=replay_domain_transform(&cipher,false,true,&entropy)?;
        let result=replay_domain_transform(&inner,false,false,&entropy);inner.fill(0);
        let mut plain=result?;
        if crate::replay_evidence::finish_recovery::witness::WitnessCandidateV1::parse(&plain).is_err(){
            plain.fill(0);return Err(Failure::CorruptRecoveryRecord);
        }
        Ok(Some(plain))
    }

    fn write_initial_finish_witness(&self,
        header:&crate::replay_evidence::finish_recovery::RecordHeaderV1<'_>,plain:&[u8])
        ->Result<(),ReplayLifecycleWriteFailureV1>{
        self.write_finish_witness_transition(header,None,plain)
    }

    fn write_finish_witness_transition(&self,
        header:&crate::replay_evidence::finish_recovery::RecordHeaderV1<'_>,previous:Option<&[u8]>,plain:&[u8])
        ->Result<(),ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        use sha2::{Digest,Sha256};
        let candidate=crate::replay_evidence::finish_recovery::witness::WitnessCandidateV1::parse(plain)
            .map_err(|_|Before(Failure::InvalidInput))?;
        match previous{
            None if candidate.stage()==0=>{},
            Some(raw)=>{
                let prior=crate::replay_evidence::finish_recovery::witness::WitnessCandidateV1::parse(raw)
                    .map_err(|_|Before(Failure::InvalidInput))?;
                if prior.stage().checked_add(1)!=Some(candidate.stage())
                    || candidate.field(11)!=Some(Sha256::digest(raw).as_slice()){
                    return Err(Before(Failure::InvalidInput));
                }
            },
            _=>return Err(Before(Failure::InvalidInput)),
        }
        // Never roll a protected stage back. Exact response retry is read-only.
        let mut existing=self.read_finish_witness(header).map_err(Before)?;
        let same=existing.as_deref()==Some(plain);
        let prior_matches=existing.as_deref()==previous;
        if let Some(raw)=&mut existing{raw.fill(0);}
        if same{return Ok(());}
        if !prior_matches{return Err(Before(Failure::ConflictingRetry));}
        let (name,entropy)=finish_witness_locator(header).map_err(Before)?;
        let mut inner=replay_domain_transform(plain,true,false,&entropy).map_err(Before)?;
        let result=replay_domain_transform(&inner,true,true,&entropy);inner.fill(0);
        let cipher=result.map_err(Before)?;
        if cipher.is_empty()||cipher.len()>MAX_CIPHERTEXT_BYTES{return Err(Before(Failure::LimitExceeded));}
        let key=WindowsRootGrantStoreV1::key(Namespace::Grant,&self.record.nonce,true,false).map_err(Before)?;
        // SAFETY: bounded protected bytes and a live existing grant key.
        if unsafe{RegSetValueExW(key.0,name.as_ptr(),0,3,cipher.as_ptr(),cipher.len() as u32)}!=0{
            return Err(Uncertain(Failure::WriteFailed));
        }
        if unsafe{RegFlushKey(key.0)}!=0{return Err(Uncertain(Failure::SyncFailed));}
        drop(key);
        let mut returned=self.read_finish_witness(header).map_err(Uncertain)?
            .ok_or(Uncertain(Failure::RereadFailed))?;
        let same=returned==plain;returned.fill(0);
        if same{Ok(())}else{Err(Uncertain(Failure::RereadMismatch))}
    }

    pub(crate) fn persist_finish_failure_capture(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,
        original:&DurableFinishIntentWitnessV1,
        captured:&super::super::replay_archive_schema::CapturedReplayMetadataFailureV1)
        ->Result<FinishFailureCaptureWitnessV1,ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,witness::WitnessCandidateV1};
        adapter.verify_finish_recovery_intent(self,intent).map_err(Before)?;
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Before(Failure::InvalidInput))?;
        let prior=WitnessCandidateV1::parse(&original.plain).map_err(|_|Before(Failure::InvalidInput))?;
        prior.check_intent(&header,intent.record_digest()).map_err(|_|Before(Failure::CustodyConflict))?;
        if prior.stage()!=0 || original.intent_digest!=*intent.record_digest(){return Err(Before(Failure::CustodyConflict));}
        let (failure_header,message)=captured.record_for(intent).map_err(|_|Before(Failure::InvalidInput))?;
        let failure_digest=RecordHeaderV1::parse(&failure_header).map_err(|_|Before(Failure::InvalidInput))?
            .check_payload([message]).map_err(|_|Before(Failure::InvalidInput))?;
        let plain=finish_witness_next_plain(&prior,None).map_err(Before)?;
        self.write_finish_witness_transition(&header,Some(&original.plain),&plain)?;
        self.verify_current(adapter).map_err(Uncertain)?;
        let witness=FinishFailureCaptureWitnessV1{plain,intent_digest:*intent.record_digest(),failure_digest};
        self.verify_finish_failure_capture(adapter,intent,captured,&witness).map_err(Uncertain)?;
        Ok(witness)
    }

    /// This gate is required by record1 INSERT. The current protected stage1,
    /// SQL Intent and actual original failure are rechecked, not just a hash.
    pub(crate) fn verify_finish_failure_capture(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,
        captured:&super::super::replay_archive_schema::CapturedReplayMetadataFailureV1,
        witness:&FinishFailureCaptureWitnessV1)->Result<(),Failure>{
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,witness::WitnessCandidateV1};
        adapter.verify_finish_recovery_intent(self,intent)?;
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Failure::InvalidInput)?;
        let candidate=WitnessCandidateV1::parse(&witness.plain).map_err(|_|Failure::InvalidInput)?;
        candidate.check_intent(&header,intent.record_digest()).map_err(|_|Failure::CustodyConflict)?;
        let (failure_header,message)=captured.record_for(intent).map_err(|_|Failure::InvalidInput)?;
        let actual=RecordHeaderV1::parse(&failure_header).map_err(|_|Failure::InvalidInput)?
            .check_payload([message]).map_err(|_|Failure::InvalidInput)?;
        if candidate.stage()!=1 || witness.intent_digest!=*intent.record_digest() || witness.failure_digest!=actual{
            return Err(Failure::CustodyConflict);
        }
        let mut actual=self.read_finish_witness(&header)?.ok_or(Failure::CustodyConflict)?;
        let same=actual==witness.plain;actual.fill(0);
        if !same{return Err(Failure::CustodyConflict);}
        self.verify_current(adapter)
    }

    pub(crate) fn persist_finish_failure_witness(&self,adapter:&super::super::SqliteAdapterV1,
        intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,
        captured:&super::super::replay_archive_schema::CapturedReplayMetadataFailureV1,
        pending:&FinishFailureCaptureWitnessV1)->Result<(),ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        use crate::replay_evidence::finish_recovery::{RecordHeaderV1,witness::WitnessCandidateV1};
        let digest=adapter.verify_finish_recovery_failure(self,intent,captured).map_err(Before)?;
        if digest!=pending.failure_digest || pending.intent_digest!=*intent.record_digest(){return Err(Before(Failure::CustodyConflict));}
        let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Before(Failure::InvalidInput))?;
        let prior=WitnessCandidateV1::parse(&pending.plain).map_err(|_|Before(Failure::InvalidInput))?;
        prior.check_intent(&header,intent.record_digest()).map_err(|_|Before(Failure::CustodyConflict))?;
        if prior.stage()!=1{return Err(Before(Failure::CustodyConflict));}
        let plain=finish_witness_next_plain(&prior,Some(&digest)).map_err(Before)?;
        self.write_finish_witness_transition(&header,Some(&pending.plain),&plain)?;
        self.verify_current(adapter).map_err(Uncertain)?;
        adapter.verify_finish_recovery_failure(self,intent,captured).map_err(Uncertain)?;
        Ok(())
    }
}

fn finish_lifecycle_failure(intent:&crate::replay_evidence::NativeFinishRecoveryIntentV1,
    stage:u8,cause:Failure,ready:bool,before_pending:bool)->super::super::CapturedReplayMetadataFailureV1{
    let message=if ready{format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 종료 Ready 보호 확인 ({cause:?})")}
        else if before_pending{format!("E_NATIVE_FINISH_BEFORE_PENDING: E_REPLAY_PENDING_NOT_WRITTEN: {cause:?}")}
        else{format!("E_REPLAY_EVIDENCE_BLOCKED: Pending 보호 기록 불확실 ({cause:?})")};
    super::super::CapturedReplayMetadataFailureV1::finish_native(stage,Some(cause),message,intent,before_pending)
}

fn finish_witness_next_plain(prior:&crate::replay_evidence::finish_recovery::witness::WitnessCandidateV1<'_>,
    failure:Option<&[u8;32]>)->Result<Vec<u8>,Failure>{
    use sha2::{Digest,Sha256};
    let mut fields=[&[][..];12];
    for (index,field) in fields.iter_mut().enumerate(){*field=prior.field(index as u8+1).ok_or(Failure::InvalidInput)?;}
    let stage=[match (prior.stage(),failure){(0,None)=>1,(1,Some(_))=>2,_=>return Err(Failure::InvalidInput)}];
    let previous:[u8;32]=Sha256::digest(prior.raw()).into();
    fields[9]=&stage;fields[10]=&previous;
    if let Some(digest)=failure{fields[8]=digest;}
    crate::replay_evidence::finish_recovery::witness::encode(&fields).map_err(|_|Failure::InvalidInput)
}

impl Drop for DurableFinishIntentWitnessV1 {
    fn drop(&mut self){self.plain.fill(0);self.intent_digest.fill(0);}
}

#[cfg(test)]
mod finish_witness_location_tests {
    use super::*;
    #[test]
    fn wo25_finish_witness_location_keeps_target_and_protection_domain_separate(){
        // Pure locator/entropy mechanics; no grant, SQL admission or DPAPI I/O.
        let nonce=[0x12;32];let first=[0xab;32];
        let (name,entropy)=finish_witness_location(&nonce,&first,2,15);
        let expected=format!("WO25FinishRecovery-{}-0000000000000002-000000000000000f",hex::encode(first));
        assert_eq!(name,utf16(&expected));
        let mut expected_entropy=FINISH_WITNESS_DOMAIN.to_vec();
        expected_entropy.extend_from_slice(&nonce);expected_entropy.extend_from_slice(&first);
        expected_entropy.extend_from_slice(&2u64.to_be_bytes());expected_entropy.extend_from_slice(&15u64.to_be_bytes());
        assert_eq!(entropy,expected_entropy);
        assert!(!entropy.starts_with(b"DDN-WO23-REPLAY-LIFECYCLE-V1\0"));
        assert_ne!(finish_witness_location(&nonce,&first,3,15), (name.clone(),entropy.clone()));
        assert_ne!(finish_witness_location(&nonce,&first,2,16), (name.clone(),entropy.clone()));
        let (other_name,other_entropy)=finish_witness_location(&[0x13;32],&first,2,15);
        assert_eq!(other_name,name,"nonce chooses the actual parent grant key, not the child value name");
        assert_ne!(other_entropy,entropy,"a different grant cannot decrypt this target");
        assert_ne!(finish_witness_location(&nonce,&[0xac;32],2,15).0,name);
    }
    #[test]
    fn wo25_finish_native_witness_next_requires_capture_before_failure_digest(){
        use crate::replay_evidence::finish_recovery::witness::{encode,WitnessCandidateV1};
        let zero=[0;32];let one=1u64.to_be_bytes();let epoch=0u64.to_be_bytes();
        let stage=[0];let intent=[8;32];let failure=[9;32];
        // Structural-only plaintext. No grant, actual cause or permit issued.
        let raw=encode(&[&zero,&zero,&one,&zero,&epoch,&one,&zero,&intent,&zero,&stage,&zero,&zero]).unwrap();
        let initial=WitnessCandidateV1::parse(&raw).unwrap();
        assert!(finish_witness_next_plain(&initial,Some(&failure)).is_err());
        let middle_raw=finish_witness_next_plain(&initial,None).unwrap();
        let middle=WitnessCandidateV1::parse(&middle_raw).unwrap();
        assert_eq!(middle.stage(),1);
        assert!(finish_witness_next_plain(&middle,None).is_err());
        assert!(finish_witness_next_plain(&middle,Some(&zero)).is_err());
        let final_raw=finish_witness_next_plain(&middle,Some(&failure)).unwrap();
        let final_candidate=WitnessCandidateV1::parse(&final_raw).unwrap();
        assert_eq!(final_candidate.stage(),2);assert_eq!(final_candidate.field(9),Some(failure.as_slice()));
        for tag in [1,2,3,4,5,6,7,8,12]{assert_eq!(final_candidate.field(tag),initial.field(tag));}
        assert!(finish_witness_next_plain(&final_candidate,Some(&failure)).is_err());
    }
}
