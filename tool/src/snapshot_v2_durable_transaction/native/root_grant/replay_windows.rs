// Included inside the existing native trust owner. No raw grant constructor
// or path/host success assertion is accepted by this child scope.
impl ReplayArchiveRootScopeV1 {
    pub(crate) fn write_replay_pending(&self,adapter:&super::super::SqliteAdapterV1,
        pending:&crate::replay_evidence::PendingReplayLifecycleV1)->Result<(),ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        self.verify_current(adapter).map_err(Before)?;
        let mut current=self.read_replay_lifecycle(pending.first_tx()).map_err(Before)?;
        let matches=match (&current,pending.previous_bytes()){
            (None,None)=>true,(Some(actual),Some(expected))=>actual==expected,_=>false,
        };
        if let Some(bytes)=&mut current{bytes.fill(0);}
        if !matches{return Err(Before(Failure::ConflictingRetry));}
        self.write_replay_lifecycle(pending.first_tx(),pending.canonical_bytes())?;
        self.verify_current(adapter).map_err(Uncertain)
    }
    pub(crate) fn write_replay_ready(&self,adapter:&super::super::SqliteAdapterV1,
        ready:&crate::replay_evidence::ReadyReplayLifecycleV1)->Result<ProtectedReplayReadyV1,Failure>{
        self.verify_current(adapter)?;
        let mut pending=self.read_replay_lifecycle(ready.first_tx())?.ok_or(Failure::CustodyConflict)?;
        let matches=pending==ready.expected_pending();pending.fill(0);
        if !matches{return Err(Failure::CustodyConflict);}
        let mut bytes=Vec::new();bytes.try_reserve_exact(ready.canonical_bytes().len()).map_err(|_|Failure::LimitExceeded)?;
        bytes.extend_from_slice(ready.canonical_bytes());
        self.write_replay_lifecycle(ready.first_tx(),ready.canonical_bytes()).map_err(ReplayLifecycleWriteFailureV1::cause)?;
        self.verify_current(adapter)?;
        let scope=ReplayArchiveRootScopeV1{record:self.record.clone(),pinned:self.pinned.clone(),lease:self.lease.clone()};
        Ok(ProtectedReplayReadyV1{scope,bytes,first_tx:*ready.first_tx()})
    }
    fn replay_entropy(&self,first_tx:&[u8;32])->Vec<u8>{
        let mut bytes=b"DDN-WO23-REPLAY-LIFECYCLE-V1\0".to_vec();
        bytes.extend_from_slice(&self.record.nonce);bytes.extend_from_slice(first_tx);bytes
    }
    fn read_replay_lifecycle(&self,first_tx:&[u8;32])->Result<Option<Vec<u8>>,Failure>{
        let key=WindowsRootGrantStoreV1::key(Namespace::Grant,&self.record.nonce,false,false)?;
        let name=utf16(&format!("WO23ReplayState-{}",hex::encode(first_tx)));
        let mut kind=0;let mut length=0;
        // SAFETY: live key and initialized output slots; no allocation before bounds.
        let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
        if status==2{return Ok(None);}
        if status!=0 || kind!=3 || length==0 || length as usize>MAX_CIPHERTEXT_BYTES{return Err(Failure::CorruptRecoveryRecord);}
        let expected=length;let mut cipher=Vec::new();cipher.try_reserve_exact(length as usize).map_err(|_|Failure::LimitExceeded)?;
        cipher.resize(length as usize,0);
        // SAFETY: buffer is allocated to the queried, bounded length.
        let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,cipher.as_mut_ptr(),&mut length)};
        if status!=0 || kind!=3 || length!=expected{return Err(Failure::RereadFailed);}
        let entropy=self.replay_entropy(first_tx);
        let mut inner=replay_domain_transform(&cipher,false,true,&entropy)?;
        let result=replay_domain_transform(&inner,false,false,&entropy);inner.fill(0);
        result.map(Some)
    }
    fn write_replay_lifecycle(&self,first_tx:&[u8;32],plain:&[u8])->Result<(),ReplayLifecycleWriteFailureV1>{
        use ReplayLifecycleWriteFailureV1::{Before,Uncertain};
        let entropy=self.replay_entropy(first_tx);
        let mut inner=replay_domain_transform(plain,true,false,&entropy).map_err(Before)?;
        let protected=replay_domain_transform(&inner,true,true,&entropy);inner.fill(0);
        let cipher=protected.map_err(Before)?;
        let key=WindowsRootGrantStoreV1::key(Namespace::Grant,&self.record.nonce,true,false).map_err(Before)?;
        let name=utf16(&format!("WO23ReplayState-{}",hex::encode(first_tx)));
        // SAFETY: bounded live buffers; existing grant key, separate REG_BINARY value.
        if unsafe{RegSetValueExW(key.0,name.as_ptr(),0,3,cipher.as_ptr(),cipher.len() as u32)}!=0{return Err(Uncertain(Failure::WriteFailed));}
        if unsafe{RegFlushKey(key.0)}!=0{return Err(Uncertain(Failure::SyncFailed));}
        drop(key);
        let mut returned=self.read_replay_lifecycle(first_tx).map_err(Uncertain)?.ok_or(Uncertain(Failure::RereadFailed))?;
        let matches=returned==plain;returned.fill(0);
        if matches{Ok(())}else{Err(Uncertain(Failure::RereadMismatch))}
    }
    fn from_source(grant:&AdmittedSourceGrantV2,adapter:&super::super::SqliteAdapterV1)->Result<Self,Failure> {
        let scope=Self{record:grant.record.base.clone(),pinned:grant.pinned.clone(),lease:grant.lease.clone()};
        scope.verify_current(adapter)?;Ok(scope)
    }
    pub(crate) fn verify_current(&self,adapter:&super::super::SqliteAdapterV1)->Result<(),Failure> {
        if !adapter.root_lease.as_ref().is_some_and(|value|std::sync::Arc::ptr_eq(value,&self.lease))
            || !adapter.pinned_root.as_ref().is_some_and(|value|std::sync::Arc::ptr_eq(value,&self.pinned)) {
            return Err(Failure::CustodyConflict);
        }
        let current=WindowsRootGrantStoreV1::read_source_grant(&self.record.nonce)?;
        let base=&current.base;
        if base!=&self.record || base.lifecycle!=Lifecycle::Active || base.sid_hash!=current_sid_hash()?
            || (base.root_file,base.root_object)!=(self.pinned.root_file,self.pinned.root_object)
            || (base.db_file,base.db_object)!=(self.pinned.db_file,self.pinned.db_object)
            || WindowsRootGrantStoreV1::read_slot(&root_fingerprint(base))?!=base.nonce {
            return Err(Failure::CustodyConflict);
        }
        self.pinned.verify_connection(&adapter.connect()?)
    }
    pub(crate) fn identity_fields(&self,adapter:&super::super::SqliteAdapterV1)
        ->Result<([u8;32],[u8;32],[u8;32],u64),Failure> {
        self.verify_current(adapter)?;
        Ok((self.record.project_hash,self.record.nonce,root_fingerprint(&self.record),self.record.generation))
    }
    pub(crate) fn protect_replay_custody(&self,adapter:&super::super::SqliteAdapterV1,
        plain:&crate::replay_evidence::ReplayCustodyPlainV1)->Result<Vec<u8>,Failure> {
        self.verify_current(adapter)?;
        let candidate=crate::replay_evidence::ReplayCustodyCandidateV1::parse(plain.canonical_bytes()).map_err(|_|Failure::InvalidInput)?;
        if !candidate.matches_root(&self.record.project_hash,&self.record.nonce,&root_fingerprint(&self.record),self.record.generation){
            return Err(Failure::CustodyConflict);
        }
        let bytes=replay_domain_transform(plain.canonical_bytes(),true,false,b"DDN-WO23-REPLAY-CUSTODY-V1\0")?;
        self.verify_current(adapter)?;Ok(bytes)
    }
    pub(crate) fn unprotect_replay_custody(&self,adapter:&super::super::SqliteAdapterV1,cipher:&[u8])->Result<Vec<u8>,Failure> {
        self.verify_current(adapter)?;
        let mut bytes=replay_domain_transform(cipher,false,false,b"DDN-WO23-REPLAY-CUSTODY-V1\0")?;
        let result=(||{
            let candidate=crate::replay_evidence::ReplayCustodyCandidateV1::parse(&bytes).map_err(|_|Failure::CorruptRecoveryRecord)?;
            if !candidate.matches_root(&self.record.project_hash,&self.record.nonce,&root_fingerprint(&self.record),self.record.generation){return Err(Failure::CustodyConflict);}
            self.verify_current(adapter)
        })();
        if let Err(cause)=result{bytes.fill(0);return Err(cause);}
        Ok(bytes)
    }
}
impl ProtectedReplayReadyV1 {
    pub(crate) fn checked_lifecycle(&self,adapter:&super::super::SqliteAdapterV1)->Result<&[u8],Failure>{
        self.scope.verify_current(adapter)?;
        let mut actual=self.scope.read_replay_lifecycle(&self.first_tx)?.ok_or(Failure::CustodyConflict)?;
        let matches=actual==self.bytes;actual.fill(0);
        if !matches{return Err(Failure::CustodyConflict);}
        self.scope.verify_current(adapter)?;Ok(&self.bytes)
    }
}
// Same audited DPAPI mechanics, but a distinct explicit entropy profile. Do
// not route old RootGrant/token protection through this child function.
fn replay_domain_transform(bytes:&[u8],protect:bool,machine:bool,entropy:&[u8])->Result<Vec<u8>,Failure> {
    if bytes.is_empty() || bytes.len()>MAX_CIPHERTEXT_BYTES || entropy.is_empty() || entropy.len()>256 {return Err(Failure::LimitExceeded);}
    let input=Blob{length:bytes.len() as u32,data:bytes.as_ptr() as *mut u8};
    let entropy=Blob{length:entropy.len() as u32,data:entropy.as_ptr() as *mut u8};
    let mut output=ProtectedOutput(Blob{length:0,data:std::ptr::null_mut()});
    // SAFETY: both borrowed buffers outlive this synchronous call. The API
    // owns its output; ProtectedOutput frees it on every return path. No UI.
    let result=unsafe {
        if protect {CryptProtectData(&input,std::ptr::null(),&entropy,std::ptr::null_mut(),std::ptr::null_mut(),1|if machine{4}else{0},&mut output.0)}
        else {CryptUnprotectData(&input,std::ptr::null_mut(),&entropy,std::ptr::null_mut(),std::ptr::null_mut(),1,&mut output.0)}
    };
    if result==0{return Err(Failure::CustodyConflict);}
    let length=output.0.length as usize;
    if output.0.data.is_null() || length==0 || length>MAX_CIPHERTEXT_BYTES{return Err(Failure::LimitExceeded);}
    let mut out=Vec::new();out.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;
    // SAFETY: DPAPI's allocation remains owned until after the bounded copy.
    out.extend_from_slice(unsafe{std::slice::from_raw_parts(output.0.data,length)});Ok(out)
}
impl FirstSourceWorldPermitV2 {
    pub(crate) fn replay_archive_scope(&self,adapter:&super::super::SqliteAdapterV1)->Result<ReplayArchiveRootScopeV1,Failure> {
        self.verify_before_world(adapter)?;
        ReplayArchiveRootScopeV1::from_source(&self.grant,adapter)
    }
}
impl NewSourceRunPermitV1 {
    pub(crate) fn replay_archive_scope(&self,adapter:&super::super::SqliteAdapterV1)->Result<ReplayArchiveRootScopeV1,Failure> {
        self.verify_before_world(adapter)?;
        ReplayArchiveRootScopeV1::from_source(&self.grant,adapter)
    }
}
impl ReadmittedFirstSourceWorldV2 {
    pub(crate) fn replay_checkpoint_proof(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<VerifiedExecutionFirstCheckpointV1,Failure>{
        // The existing historical consumer covers both initial and restarted
        // mixed histories. It never falls back from old source A to current B.
        let (first,receipt)=self.first_payload_and_current_receipt(adapter,context)?;
        if first.binding.pesi!=self.payload.binding.pesi{return Err(Failure::CustodyConflict);}
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&self.payload.snapshot,&self.payload.binding.pesi)?;
        let scope=ReplayArchiveRootScopeV1::from_source(&self._grant,adapter)?;
        Ok(VerifiedExecutionFirstCheckpointV1{scope,first_tx:first.digest,
            checkpoint_tx:self.payload.digest,receipt,pesi:self.payload.binding.pesi,checkpoint_next_madi:layout.world.next_tick})
    }
}
impl super::super::SqliteAdapterV1 {
    pub(crate) fn revalidate_replay_scope_current_world(&mut self,scope:&ReplayArchiveRootScopeV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<ReadmittedFirstSourceWorldV2,Failure>{
        scope.verify_current(self)?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&scope.record.nonce)?;
        let current=self.readmit_source_world_pinned_context(record,&context.source_context(),false)?;
        scope.verify_current(self)?;Ok(current)
    }
}
impl VerifiedExecutionFirstCheckpointV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) fn anchor_madi(&self)->u64{self.checkpoint_next_madi}
    pub(crate) fn into_checkpoint_plan_scope(self,adapter:&super::super::SqliteAdapterV1)
        ->Result<(ReplayArchiveRootScopeV1,Vec<u8>),Failure>{
        let lifecycle=self.read_current_lifecycle(adapter)?;
        Ok((self.scope,lifecycle))
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn into_new_record_plan_scope(self,
        adapter:&super::super::SqliteAdapterV1)->Result<(ReplayArchiveRootScopeV1,Vec<u8>),Failure>{
        let lifecycle=self.read_lifecycle_for(adapter,ReplayArchiveReadKindV1::TerminalForNewRecord)?;
        Ok((self.scope,lifecycle))
    }
    pub(crate) fn replay_progress_present(&self,adapter:&super::super::SqliteAdapterV1)->Result<bool,Failure>{
        self.scope.verify_current(adapter)?;
        let protected=self.scope.read_replay_lifecycle(&self.first_tx)?;
        let db=adapter.connect()?;
        let tables:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name='wo23_replay_controls_v1'",[],|r|r.get(0)).map_err(super::super::database_error)?;
        let count:i64=if tables==0{0}else{db.query_row("SELECT count(*) FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1",
            rusqlite::params![self.first_tx.as_slice()],|r|r.get(0)).map_err(super::super::database_error)?};
        self.scope.verify_current(adapter)?;
        // Only both absent selects legacy normal storage. Any partial child
        // is present-but-invalid and must go through strict admission/refusal.
        Ok(protected.is_some() || count!=0)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn read_current_lifecycle(&self,
        adapter:&super::super::SqliteAdapterV1)->Result<Vec<u8>,Failure>{
        self.read_lifecycle_for(adapter,ReplayArchiveReadKindV1::Running)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn read_lifecycle_for(&self,
        adapter:&super::super::SqliteAdapterV1,kind:ReplayArchiveReadKindV1)->Result<Vec<u8>,Failure>{
        use sha2::{Digest,Sha256};
        self.scope.verify_current(adapter)?;
        let bytes=self.scope.read_replay_lifecycle(&self.first_tx)?.ok_or(Failure::UnsupportedDurability)?;
        let parsed=crate::replay_evidence::ReplayLifecycleCandidateV1::parse(&bytes).map_err(|_|Failure::CorruptRecoveryRecord)?;
        let (_,nonce,fingerprint,generation)=self.scope.identity_fields(adapter)?;
        let receipt_hash:[u8;32]=Sha256::digest(&self.receipt).into();
        let state=parsed.payload(9).and_then(|v|v.first()).copied().ok_or(Failure::CorruptRecoveryRecord)?;
        let count=u64::from_be_bytes(parsed.payload(15).ok_or(Failure::CorruptRecoveryRecord)?.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?);
        if parsed.payload(1)!=Some(nonce.as_slice()) || parsed.payload(2)!=Some(fingerprint.as_slice())
            || parsed.payload(3)!=Some(generation.to_be_bytes().as_slice()) || parsed.payload(4)!=Some(self.pesi.as_slice())
            || parsed.payload(5)!=Some(self.first_tx.as_slice()) || parsed.payload(8)!=Some(&[1][..])
            || !kind.accepts_state(state) || parsed.payload(10)!=Some(self.checkpoint_tx.as_slice())
            || parsed.payload(11)!=Some(receipt_hash.as_slice())
            || !kind.accepts_count(state,count,self.checkpoint_next_madi)
            || !parsed.payload(17).is_some_and(|value|kind.accepts_end(state,value[0])) {
            return Err(Failure::CustodyConflict);
        }
        self.scope.verify_current(adapter)?;Ok(bytes)
    }
}
