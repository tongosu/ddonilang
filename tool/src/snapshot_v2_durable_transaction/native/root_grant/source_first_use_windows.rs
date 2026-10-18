// Included inside the existing Windows protected owner. Same Record/RootSlot,
// same DPAPI layers/flush/fresh readback. No new registry namespace or JSON.
use super::first_use::{Grant2,FirstUse,SourceAnchor};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as SourceOnly;
fn source_failure(phase:&'static str,cause:Failure)->SourceGrantFailureV2{SourceGrantFailureV2{phase,cause}}

impl WindowsRootGrantStoreV1 {
    fn write_next_source_pending(grant:&Grant2,previous:&AttemptRecord,pending:&AttemptRecord)->Result<(),Failure>{
        if Self::read_source_grant(&grant.base.nonce)?!=*grant||grant.base.lifecycle!=Lifecycle::Active
            ||!matches!(grant.state,FirstUse::Started(_))||Self::read_attempt(&grant.base)?!=*previous
            ||previous.mode!=AttemptMode::RecordWitness||pending.mode!=AttemptMode::Pending
            ||pending.nonce!=grant.base.nonce||pending.generation!=grant.base.generation
            ||previous.epoch.checked_add(1)!=Some(pending.epoch)||pending.transaction==previous.transaction {
            return Err(Failure::CustodyConflict);
        }
        pending.validate()?;let mut bytes=pending.encode()?;
        let result=Self::write_plaintext(Namespace::Attempt,&grant.base.nonce,&bytes,false);bytes.fill(0);result?;
        if Self::read_source_grant(&grant.base.nonce)?!=*grant||Self::read_attempt(&grant.base)?!=*pending{return Err(Failure::RereadMismatch);}
        Ok(())
    }
    fn write_source_pending(grant:&Grant2,pending:&AttemptRecord)->Result<(),Failure>{
        if Self::read_source_grant(&grant.base.nonce)?!=*grant||grant.base.lifecycle!=Lifecycle::Active
            || !matches!(grant.state,FirstUse::Started(_))||pending.nonce!=grant.base.nonce
            ||pending.generation!=grant.base.generation||pending.epoch!=1||pending.mode!=AttemptMode::Pending{
            return Err(Failure::CustodyConflict);
        }
        pending.validate()?;Self::require_no_source_attempt(grant)?;
        let mut bytes=pending.encode()?;
        let result=Self::write_plaintext(Namespace::Attempt,&grant.base.nonce,&bytes,false);
        bytes.fill(0);result?;
        if Self::read_source_grant(&grant.base.nonce)?!=*grant||Self::read_attempt(&grant.base)?!=*pending{
            return Err(Failure::RereadMismatch);
        }
        Ok(())
    }
    fn read_source_grant(nonce:&[u8;32])->Result<Grant2,Failure>{
        let mut bytes=Self::read_plaintext(Namespace::Grant,nonce)?;
        let decoded=Grant2::decode(&bytes);bytes.fill(0);let decoded=decoded?;
        if decoded.base.nonce!=*nonce{return Err(Failure::CustodyConflict);}Ok(decoded)
    }
    fn write_source_grant_new(record:&Grant2)->Result<(),Failure>{
        let mut bytes=record.encode()?;
        let result=Self::write_plaintext(Namespace::Grant,&record.base.nonce,&bytes,true);
        bytes.fill(0);result
    }
    fn consume_source_grant(old:&Grant2,new:&Grant2)->Result<(),Failure>{
        // Open an existing writable key only. Missing/deleted Record must not
        // be healed by RegCreateKeyEx during consumption.
        let path=utf16(&format!("{}{}",Namespace::Grant.prefix(),hex::encode(old.base.nonce)));
        let mut handle=null_mut();
        if unsafe{RegOpenKeyExW(hkcu(),path.as_ptr(),0,0x103,&mut handle)}!=0||handle.is_null(){return Err(Failure::CustodyConflict);}
        let key=Key(handle);let mut expected=old.encode()?;
        let mut previous=Self::read_plaintext_on_key(&key,"Record")?;
        let matches=previous==expected;previous.fill(0);expected.fill(0);
        if !matches{return Err(Failure::ConflictingRetry);}
        let mut bytes=new.encode()?;let ciphertext=protect(&bytes)?;let name=utf16("Record");
        let written=(||{
            #[cfg(test)] inject_io_failure(Namespace::Grant,IoStep::Set)?;
            if unsafe{RegSetValueExW(key.0,name.as_ptr(),0,3,ciphertext.as_ptr(),ciphertext.len() as u32)}!=0{return Err(Failure::WriteFailed);}
            #[cfg(test)] inject_io_failure(Namespace::Grant,IoStep::Flush)?;
            if unsafe{RegFlushKey(key.0)}!=0{return Err(Failure::SyncFailed);}
            drop(key);
            #[cfg(test)] inject_io_failure(Namespace::Grant,IoStep::FreshRead)?;
            let mut read=Self::read_plaintext(Namespace::Grant,&old.base.nonce)?;
            let matches=read==bytes;read.fill(0);
            if matches{Ok(())}else{Err(Failure::RereadMismatch)}
        })();
        bytes.fill(0);written
    }
    fn require_no_source_attempt(record:&Grant2)->Result<(),Failure>{
        let key=Self::key(Namespace::Grant,&record.base.nonce,false,false)?;
        let mut kind=0;let mut length=0;let name=utf16("Attempt");
        let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
        if status==2{Ok(())}else{Err(Failure::RecoveryRequired)}
    }
}

impl AdmittedSourceGrantV2 {
    pub(crate) fn locator(&self)->String{hex::encode(self.record.base.nonce)}
    fn verify_root(&self,adapter:&super::super::SqliteAdapterV1,project:&[u8])->Result<(),Failure>{
        use sha2::{Digest,Sha256};
        if !adapter.root_lease.as_ref().is_some_and(|lease|std::sync::Arc::ptr_eq(lease,&self.lease))
            ||!adapter.pinned_root.as_ref().is_some_and(|pin|std::sync::Arc::ptr_eq(pin,&self.pinned)) {
            return Err(Failure::CustodyConflict);
        }
        let current=WindowsRootGrantStoreV1::read_source_grant(&self.record.base.nonce)?;
        let base=&current.base;
        if current!=self.record||base.lifecycle!=Lifecycle::Active||base.sid_hash!=current_sid_hash()?
            ||base.project_hash!=<[u8;32]>::from(Sha256::digest(project))
            ||(base.root_file,base.root_object)!=(self.pinned.root_file,self.pinned.root_object)
            ||(base.db_file,base.db_object)!=(self.pinned.db_file,self.pinned.db_object)
            ||WindowsRootGrantStoreV1::read_slot(&root_fingerprint(base))?!=base.nonce {
            return Err(Failure::CustodyConflict);
        }
        self.pinned.verify_connection(&adapter.connect()?)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn verify_initial_source(
        &self,adapter:&super::super::SqliteAdapterV1,source:&SourceOnly)->Result<(),Failure>{
        self.verify_root(adapter,source.project())?;
        if self.record.state!=FirstUse::Unused||!source.parent().is_empty()
            ||self.record.initial.revision!=source.revision()||self.record.initial.integrity!=*source.integrity()
            ||self.record.base.registration_pesi!=*source.pesi(){return Err(Failure::CustodyConflict);}
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.record)
    }
    fn checked_chain(&self,adapter:&super::super::SqliteAdapterV1,project:&[u8])
        ->Result<super::super::source_chain::SourceChain,Failure>{
        self.verify_root(adapter,project)?;
        let chain=adapter.read_source_chain(project)?;
        if self.record.initial.revision!=chain.initial.revision()
            ||self.record.initial.integrity!=*chain.initial.integrity()
            ||self.record.base.registration_pesi!=*chain.initial.pesi(){return Err(Failure::CustodyConflict);}
        self.verify_root(adapter,project)?;Ok(chain)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn verify_source_successor(
        &self,adapter:&super::super::SqliteAdapterV1,next:&SourceOnly)->Result<[u8;32],Failure>{
        if self.record.state!=FirstUse::Unused{return Err(Failure::RecoveryRequired);}
        let chain=self.checked_chain(adapter,next.project())?;
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.record)?;
        if next.parent()!=chain.latest.revision()||next.profile()!=chain.latest.profile(){return Err(Failure::StaleRevision);}
        match (chain.latest.owned().registration(),next.owned().registration()){
            (Some(old),Some(new))=>new.verify_retained_from(old).map_err(|_|Failure::CustodyConflict)?,
            (Some(_),None)=>return Err(Failure::CustodyConflict),
            _=>{},
        }
        Ok(*chain.latest.integrity())
    }
    pub(crate) fn prepare_source_successor(&self,adapter:&super::super::SqliteAdapterV1,
        artifact:&str,revision:&[u8],expected_parent:&[u8],expected_integrity:&[u8;32])
        ->Result<super::super::source_revision::SourceEditV2,String>{
        use super::super::source_revision::SourceEditV2;
        if self.record.state!=FirstUse::Unused{return Err("E_SOURCE_SUCCESSOR_STARTED: 이미 사용을 시작한 원문 머리는 바꾸지 않습니다".into());}
        let project=crate::authored_project::open(artifact.as_bytes())
            .map_err(|_|"E_SOURCE_SUCCESSOR_ARTIFACT")?;
        let project=hex::decode(project.authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_SOURCE_SUCCESSOR_PROJECT")?;
        let previous=self.current_source(adapter,&project).map_err(|cause|format!("E_SOURCE_SUCCESSOR_PARENT: {cause:?}"))?;
        if previous.revision()!=expected_parent||previous.integrity()!=expected_integrity {
            return Err("E_SOURCE_SUCCESSOR_STALE: 편집을 시작한 원문 판과 현재 저장 머리가 다릅니다".into());
        }
        let raw=crate::authored_project::operate("source_prepare",&serde_json::json!({
            "artifact_json_utf8":artifact,"expected_parent_hex":hex::encode(previous.revision()),
            "revision_hex":hex::encode(revision),"previous_source_artifact_utf8":previous.artifact()?,
        }).to_string())?;
        let next=SourceOnly::from_artifact(&raw)?;
        if next.artifact()?==previous.artifact()?{return Ok(SourceEditV2::Unchanged(next));}
        self.verify_source_successor(adapter,&next).map_err(|cause|format!("E_SOURCE_SUCCESSOR_BINDING: {cause:?}"))?;
        Ok(SourceEditV2::Successor(next))
    }
    pub(crate) fn current_source(&self,adapter:&super::super::SqliteAdapterV1,project:&[u8])->Result<SourceOnly,Failure>{
        if self.record.state!=FirstUse::Unused{return Err(Failure::RecoveryRequired);}
        let chain=self.checked_chain(adapter,project)?;
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.record)?;Ok(chain.latest)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn verify_saved_source(
        &self,adapter:&super::super::SqliteAdapterV1,source:&SourceOnly)->Result<(),Failure>{
        if self.record.state!=FirstUse::Unused{return Err(Failure::RecoveryRequired);}
        let chain=self.checked_chain(adapter,source.project())?;
        if chain.latest.revision()!=source.revision()||chain.latest.integrity()!=source.integrity()
            ||chain.latest.payload()!=source.payload()||chain.latest.envelope()!=source.envelope(){return Err(Failure::RereadMismatch);}
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.record)
    }
    pub(crate) fn consume_first_source_world(mut self,adapter:&super::super::SqliteAdapterV1,project:&[u8],expected_revision:&[u8],expected_integrity:&[u8;32])
        ->Result<FirstSourceWorldPermitV2,SourceGrantFailureV2>{
        let phase=|cause|source_failure("before-consume",cause);
        if self.record.state!=FirstUse::Unused{return Err(phase(Failure::RecoveryRequired));}
        let chain=self.checked_chain(adapter,project).map_err(phase)?;
        if chain.latest.revision()!=expected_revision||chain.latest.integrity()!=expected_integrity{return Err(phase(Failure::StaleRevision));}
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.record).map_err(phase)?;
        let next=self.record.started_candidate(SourceAnchor{revision:chain.latest.revision().to_vec(),integrity:*chain.latest.integrity()}).map_err(phase)?;
        WindowsRootGrantStoreV1::consume_source_grant(&self.record,&next)
            .map_err(|cause|source_failure("consume-unconfirmed",cause))?;
        self.record=next;
        let after=self.checked_chain(adapter,project).map_err(|cause|source_failure("after-consume",cause))?;
        if after.latest.artifact().map_err(|_|source_failure("after-consume",Failure::CustodyConflict))?
            !=chain.latest.artifact().map_err(|_|source_failure("after-consume",Failure::CustodyConflict))? {
            return Err(source_failure("after-consume",Failure::CustodyConflict));
        }
        Ok(FirstSourceWorldPermitV2{grant:self,source:after.latest})
    }
}

impl FirstSourceWorldPermitV2 {
    pub(crate) fn locator(&self)->String{self.grant.locator()}
    pub(crate) fn source(&self)->&SourceOnly{&self.source}
    pub(crate) fn check_actual_world_request(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        request:&crate::snapshot_v2_durable_transaction::retained_attempt::ActualPreparedRequestV1)->Result<(),Failure>{
        self.verify_before_world(adapter)?;
        self.check_request_source_fields(context,request)
    }
    fn check_request_source_fields(&self,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        request:&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<(),Failure>{
        request.check_source(context)?;
        if request.binding().project!=self.source.project()
            || request.binding().expected_parent!=self.source.revision()
            || request.binding().execution_profile!=self.source.profile()
            || crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(context.owned())
                .map_err(|_|Failure::CustodyConflict)?.as_bytes()!=self.source.payload(){
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
    pub(crate) fn start_first_world_from_saved_source(self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<SourceWorldAttemptStartV2,Failure>{
        self.check_actual_world_request(adapter,context,&request)?;
        let digest=request.binding().transaction_digest()?;
        let mut token=[0;33];token[0]=1;token[1..].copy_from_slice(&digest);
        let attempt=AttemptRecord::initial_pending(&self.grant.record.base,digest,request.raw(),&token)?;
        WindowsRootGrantStoreV1::write_source_pending(&self.grant.record,&attempt)?;
        let start=SourceWorldAttemptStartV2{permit:self,attempt,request};
        start.check_current(adapter,context)?;Ok(start)
    }
    pub(crate) fn verify_before_world(&self,adapter:&super::super::SqliteAdapterV1)->Result<(),Failure>{
        let chain=self.grant.checked_chain(adapter,self.source.project())?;
        match &self.grant.record.state {
            FirstUse::Started(parent) if parent.revision==self.source.revision()&&parent.integrity==*self.source.integrity()=>{},
            _=>return Err(Failure::CustodyConflict),
        }
        if chain.latest.artifact().map_err(|_|Failure::CustodyConflict)?!=self.source.artifact().map_err(|_|Failure::CustodyConflict)? {
            return Err(Failure::CustodyConflict);
        }
        WindowsRootGrantStoreV1::require_no_source_attempt(&self.grant.record)
    }
}

impl SourceWorldAttemptStartV2 {
    pub(crate) fn retain_source_witness(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        sql:&super::super::source_world::SourceSqlRetainedV2)->Result<RetainedAttemptWitnessV1,Failure>{
        use sha2::{Digest,Sha256};
        if !sql.start().first_is(self){return Err(Failure::CustodyConflict);}
        self.check_current(adapter,context)?;sql.revalidate(adapter)?;
        let mut witness=self.attempt.clone();witness.mode=AttemptMode::RecordWitness;
        witness.record_hash=Sha256::digest(sql.record()).into();witness.validate()?;
        self.check_current(adapter,context)?;
        let mut bytes=witness.encode()?;
        let result=WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&self.permit.grant.record.base.nonce,&bytes,false);
        bytes.fill(0);result?;
        self.permit.grant.verify_root(adapter,self.source().project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.permit.grant.record.base)?!=witness{return Err(Failure::RereadMismatch);}
        sql.revalidate(adapter)?;Ok(RetainedAttemptWitnessV1{_attempt:witness})
    }
    pub(crate) fn request(&self)->&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1{&self.request}
    pub(crate) fn source(&self)->&SourceOnly{&self.permit.source}
    pub(crate) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{
        self.permit.grant.verify_root(adapter,self.source().project())?;
        self.permit.check_request_source_fields(context,&self.request)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.permit.grant.record.base)?!=self.attempt
            ||self.attempt.mode!=AttemptMode::Pending{return Err(Failure::CustodyConflict);}
        let db=adapter.connect()?;
        let source=super::super::source_chain::read(&db,self.source().project(),self.source().revision())?;
        if source.artifact().map_err(|_|Failure::CustodyConflict)?!=self.source().artifact().map_err(|_|Failure::CustodyConflict)?{
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
}

impl ReadmittedFirstSourceWorldV2 {
    /// Consumes fresh actual first-W admission, not its JSON projection. The
    /// supplied next request must come from the actual continuation encoder.
    pub(crate) fn start_next_source_world(self,adapter:&super::super::SqliteAdapterV1,
        previous:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        next:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<NextSourceWorldAttemptStartV1,Failure>{
        let Self{_grant:grant,_attempt:old,payload,history:_}=self;
        grant.verify_root(adapter,&payload.binding.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?!=old||old.mode!=AttemptMode::RecordWitness {
            return Err(Failure::RecoveryRequired);
        }
        let anchor=match &grant.record.state{FirstUse::Started(anchor)=>anchor,_=>return Err(Failure::CustodyConflict)};
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let chain=super::super::source_chain::read_retained_source_ancestry(&db,&payload.binding.project,&anchor.revision)?;
        let (actual,_)=super::super::source_world::check_cold_source_world_history(&db,&chain.latest,&previous.source_context(),
            &old.transaction,&old.request_hash,&old.token_hash,&old.record_hash,old.epoch)?;
        if actual!=payload{return Err(Failure::CustodyConflict);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        request.check_source(next)?;let binding=request.binding();
        if binding.project!=payload.binding.project||binding.pesi!=payload.binding.pesi||binding.execution_profile!=payload.binding.execution_profile
            ||binding.expected_parent!=payload.binding.new_revision||binding.new_revision==payload.binding.new_revision
            ||next.owned().project().save().map_err(|_|Failure::CustodyConflict)?!=previous.owned().project().save().map_err(|_|Failure::CustodyConflict)? {
            return Err(Failure::CustodyConflict);
        }
        let digest=request.payload().digest;let mut token=[0;33];token[0]=1;token[1..].copy_from_slice(&digest);
        let mut pending=AttemptRecord::initial_pending(&grant.record.base,digest,request.raw(),&token)?;
        pending.epoch=old.epoch.checked_add(1).ok_or(Failure::LimitExceeded)?;
        grant.verify_root(adapter,&binding.project)?;
        WindowsRootGrantStoreV1::write_next_source_pending(&grant.record,&old,&pending)?;
        let previous=super::super::StoredHeadV1{project:payload.binding.project,revision:payload.binding.new_revision,
            integrity:payload.binding.revision_integrity,digest:payload.digest};
        let start=NextSourceWorldAttemptStartV1{grant,source:chain.latest,attempt:pending,request,previous};
        start.check_current(adapter,next)?;Ok(start)
    }
}
impl NextSourceWorldAttemptStartV1 {
    pub(crate) fn request(&self)->&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1{&self.request}
    pub(crate) fn source(&self)->&SourceOnly{&self.source}
    pub(crate) fn previous(&self)->&super::super::StoredHeadV1{&self.previous}
    pub(crate) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{
        self.grant.verify_root(adapter,&self.previous.project)?;self.request.check_source(context)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=self.attempt||self.attempt.mode!=AttemptMode::Pending
            ||self.request.binding().expected_parent!=self.previous.revision||self.request.binding().project!=self.previous.project
            ||self.request.binding().pesi!=*self.source.pesi(){return Err(Failure::CustodyConflict);}
        let source=super::super::source_chain::read(&adapter.connect()?,self.source.project(),self.source.revision())?;
        if source.artifact().map_err(|_|Failure::CustodyConflict)?!=self.source.artifact().map_err(|_|Failure::CustodyConflict)?{return Err(Failure::CustodyConflict);}
        Ok(())
    }
    pub(crate) fn retain_source_witness(&self,adapter:&super::super::SqliteAdapterV1,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        sql:&super::super::source_world::SourceSqlRetainedV2)->Result<RetainedAttemptWitnessV1,Failure>{
        use sha2::{Digest,Sha256};
        if !sql.start().next_is(self){return Err(Failure::CustodyConflict);}
        self.check_current(adapter,context)?;sql.revalidate(adapter)?;
        let mut witness=self.attempt.clone();witness.mode=AttemptMode::RecordWitness;witness.record_hash=Sha256::digest(sql.record()).into();witness.validate()?;
        self.check_current(adapter,context)?;let mut bytes=witness.encode()?;
        let result=WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&self.grant.record.base.nonce,&bytes,false);bytes.fill(0);result?;
        self.grant.verify_root(adapter,&self.previous.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=witness{return Err(Failure::RereadMismatch);}
        sql.revalidate(adapter)?;Ok(RetainedAttemptWitnessV1{_attempt:witness})
    }
}

impl super::super::SqliteAdapterV1 {
    /// Physical cache locator only. Consumers must still admit the exact
    /// artifact against this protected Witness before restoring any world.
    pub(crate) fn current_source_world_digest(&mut self,locator:&str,project:&[u8])->Result<[u8;32],Failure>{
        if self.schema_profile!=super::super::NativeSchemaProfile::SourceRevision||locator.len()!=64{return Err(Failure::InvalidInput);}
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&nonce)?;
        if !matches!(record.state,FirstUse::Started(_)){return Err(Failure::RecoveryRequired);}
        self.pin_objects_for_grant(false)?;
        let grant=AdmittedSourceGrantV2{record,pinned:self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone(),
            lease:self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone()};
        grant.verify_root(self,project)?;
        let attempt=WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?;
        if attempt.mode!=AttemptMode::RecordWitness{return Err(Failure::RecoveryRequired);}
        grant.verify_root(self,project)?;
        if WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?!=attempt{return Err(Failure::CustodyConflict);}
        Ok(attempt.transaction)
    }
    pub(crate) fn readmit_first_source_world(&mut self,locator:&str,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<ReadmittedFirstSourceWorldV2,Failure>{
        self.readmit_first_source_world_source_context(locator,&context.source_context())
    }
    pub(crate) fn readmit_first_source_world_source_context(&mut self,locator:&str,
        context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>)
        ->Result<ReadmittedFirstSourceWorldV2,Failure>{
        self.readmit_source_world_source_context(locator,context,true)
    }
    pub(crate) fn readmit_current_source_world_source_context(&mut self,locator:&str,
        context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>)
        ->Result<ReadmittedFirstSourceWorldV2,Failure>{
        self.readmit_source_world_source_context(locator,context,false)
    }
    fn readmit_source_world_source_context(&mut self,locator:&str,
        context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>,first_only:bool)
        ->Result<ReadmittedFirstSourceWorldV2,Failure>{
        if self.schema_profile!=super::super::NativeSchemaProfile::SourceRevision||locator.len()!=64
            ||!locator.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err(Failure::InvalidInput);}
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&nonce)?;
        let parent=match &record.state{FirstUse::Started(parent)=>parent.clone(),_=>return Err(Failure::RecoveryRequired)};
        self.pin_objects_for_grant(false)?;
        let grant=AdmittedSourceGrantV2{record,pinned:self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone(),
            lease:self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone()};
        let project=hex::decode(context.owned().project().authored_identity.logical_dependency_id.as_str()).map_err(|_|Failure::CustodyConflict)?;
        grant.verify_root(self,&project)?;
        let mut raw=grant.record.encode()?;let result=WindowsRootGrantStoreV1::confirm_current(Namespace::Grant,&nonce,&raw);raw.fill(0);result?;
        let mut slot=encode_slot(&nonce)?;let result=WindowsRootGrantStoreV1::confirm_current(Namespace::RootSlot,&root_fingerprint(&grant.record.base),&slot);slot.fill(0);result?;
        let attempt=WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?;
        if attempt.mode!=AttemptMode::RecordWitness||(first_only&&attempt.epoch!=1){return Err(Failure::RecoveryRequired);}
        self.confirm_container_durable()?;
        let db=self.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let chain=super::super::source_chain::read_retained_source_ancestry(&db,&project,&parent.revision)?;
        if chain.latest.integrity()!=&parent.integrity||chain.initial.revision()!=grant.record.initial.revision
            ||chain.initial.integrity()!=&grant.record.initial.integrity||chain.initial.pesi()!=&grant.record.base.registration_pesi{
            return Err(Failure::CustodyConflict);
        }
        let (payload,history)=if first_only {
            let payload=super::super::source_world::check_cold_first_world(&db,&chain.latest,context,&attempt.transaction,
                &attempt.request_hash,&attempt.token_hash,&attempt.record_hash)?;
            let history=vec![payload.digest];(payload,history)
        }else{
            super::super::source_world::check_cold_source_world_history(&db,&chain.latest,context,&attempt.transaction,
                &attempt.request_hash,&attempt.token_hash,&attempt.record_hash,attempt.epoch)?
        };
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        grant.verify_root(self,&project)?;
        if WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?!=attempt{return Err(Failure::CustodyConflict);}
        Ok(ReadmittedFirstSourceWorldV2{_grant:grant,_attempt:attempt,payload,history})
    }
    pub(crate) fn register_source_root_grant(&mut self,source:&SourceOnly)->Result<AdmittedSourceGrantV2,SourceGrantFailureV2>{
        use sha2::{Digest,Sha256};
        let phase=|cause|source_failure("prepare-registration",cause);
        if self.schema_profile!=super::super::NativeSchemaProfile::SourceRevision||!source.parent().is_empty(){return Err(phase(Failure::UnsupportedDurability));}
        if source.owned().registration().is_some_and(|draft|{
            let start=b"ddn.vc04.atom-source-binding.v1\0".len();
            draft.descriptor().get(start..start+2)!=Some(&[0,3])
        }) {return Err(phase(Failure::UnsupportedDurability));}
        let fresh=self.fresh_container.get_mut().take().ok_or_else(||phase(Failure::CustodyConflict))?;
        self.pin_objects_for_grant(true).map_err(phase)?;
        let pinned=self.pinned_root.as_ref().ok_or_else(||phase(Failure::UnsupportedDurability))?.clone();
        fresh.verify_pinned(&pinned).map_err(phase)?;
        let lease=self.root_lease.as_ref().ok_or_else(||phase(Failure::UnsupportedDurability))?.clone();
        let db=self.connect().map_err(phase)?;
        for table in ["vc04_prepared_payloads","vc04_project_revisions","vc04_project_heads","vc04_recovery_custody_v1"] {
            let count:i64=db.query_row(&format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 1)"),[],|r|r.get(0))
                .map_err(|_|phase(Failure::CustodyConflict))?;
            if count!=0{return Err(phase(Failure::CustodyConflict));}
        }
        drop(db);let mut nonce=[0;32];getrandom::fill(&mut nonce).map_err(|_|phase(Failure::UnsupportedDurability))?;
        let base=GrantRecord{nonce,sid_hash:current_sid_hash().map_err(phase)?,project_hash:Sha256::digest(source.project()).into(),
            registration_pesi:*source.pesi(),root_file:pinned.root_file,root_object:pinned.root_object,db_file:pinned.db_file,db_object:pinned.db_object,
            lifecycle:Lifecycle::Active,generation:0};
        let record=Grant2{base,initial:SourceAnchor{revision:source.revision().to_vec(),integrity:*source.integrity()},state:FirstUse::Unused};
        let phase=|cause|source_failure("protected-registration",cause);
        WindowsRootGrantStoreV1::ensure_slot_absent(&root_fingerprint(&record.base)).map_err(phase)?;
        WindowsRootGrantStoreV1::write_source_grant_new(&record).map_err(phase)?;
        WindowsRootGrantStoreV1::write_slot_new(&root_fingerprint(&record.base),&nonce).map_err(phase)?;
        let grant=AdmittedSourceGrantV2{record,pinned,lease};
        self.save_initial_source_revision_bound(source,Some(&grant)).map_err(|failure|{
            use super::super::source_revision::SourceSaveFailure::*;
            match failure{BeforeCommit(cause)=>source_failure("source-save-before-commit",cause),
                AfterCommitUnconfirmed(cause)=>source_failure("source-save-unconfirmed",cause)}
        })?;
        grant.checked_chain(self,source.project()).map_err(|cause|source_failure("source-readback",cause))?;
        Ok(grant)
    }
    pub(crate) fn confirm_unused_source_root_grant(&mut self,locator:&str,project:&[u8])
        ->Result<AdmittedSourceGrantV2,SourceGrantFailureV2>{
        let phase=|cause|source_failure("cold-confirm",cause);
        if locator.len()!=64||!locator.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {return Err(phase(Failure::InvalidInput));}
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|phase(Failure::InvalidInput))?.try_into().map_err(|_|phase(Failure::InvalidInput))?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&nonce).map_err(phase)?;
        if record.state!=FirstUse::Unused{return Err(phase(Failure::RecoveryRequired));}
        self.pin_objects_for_grant(false).map_err(phase)?;
        let grant=AdmittedSourceGrantV2{record,pinned:self.pinned_root.as_ref().ok_or_else(||phase(Failure::UnsupportedDurability))?.clone(),
            lease:self.root_lease.as_ref().ok_or_else(||phase(Failure::UnsupportedDurability))?.clone()};
        grant.checked_chain(self,project).map_err(phase)?;
        WindowsRootGrantStoreV1::require_no_source_attempt(&grant.record).map_err(phase)?;
        let mut raw=grant.record.encode().map_err(phase)?;
        let result=WindowsRootGrantStoreV1::confirm_current(Namespace::Grant,&nonce,&raw);raw.fill(0);result.map_err(phase)?;
        let mut slot=encode_slot(&nonce).map_err(phase)?;
        let result=WindowsRootGrantStoreV1::confirm_current(Namespace::RootSlot,&root_fingerprint(&grant.record.base),&slot);slot.fill(0);result.map_err(phase)?;
        grant.checked_chain(self,project).map_err(phase)?;Ok(grant)
    }
}

impl ReadmittedFirstSourceWorldV2 {
    pub(crate) fn payload(&self)->&super::super::StoredPayloadV1{&self.payload}
    pub(crate) fn current_receipt(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<Vec<u8>,Failure>{
        self.historical_payload(adapter,self.history.len().checked_sub(1).ok_or(Failure::RecoveryRequired)?,context)?;
        crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(&self.payload.binding,adapter.capability()?.as_bytes())
    }
    pub(crate) fn historical_payload(&self,adapter:&super::super::SqliteAdapterV1,index:usize,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<super::super::StoredPayloadV1,Failure>{
        self._grant.verify_root(adapter,&self.payload.binding.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self._grant.record.base)?!=self._attempt{return Err(Failure::CustodyConflict);}
        let anchor=match &self._grant.record.state{FirstUse::Started(anchor)=>anchor,_=>return Err(Failure::RecoveryRequired)};
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let source=super::super::source_chain::read_retained_source_ancestry(&db,&self.payload.binding.project,&anchor.revision)?;
        let (current,history)=super::super::source_world::check_cold_source_world_history(&db,&source.latest,&context.source_context(),
            &self._attempt.transaction,&self._attempt.request_hash,&self._attempt.token_hash,&self._attempt.record_hash,self._attempt.epoch)?;
        if current!=self.payload||history!=self.history{return Err(Failure::CustodyConflict);}
        let digest=history.get(index).ok_or(Failure::InvalidInput)?;
        let payload=super::super::source_world::read_source_history_payload(&db,digest)?;
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;
        self._grant.verify_root(adapter,&self.payload.binding.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self._grant.record.base)?!=self._attempt{return Err(Failure::CustodyConflict);}
        Ok(payload)
    }
}

#[cfg(test)]
mod source_first_use_tests {
    use super::*;
    fn fixture()->(std::path::PathBuf,crate::authored_project::ProjectArtifact){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let mut nonce=[0;32];getrandom::fill(&mut nonce).unwrap();
        let directory=std::env::temp_dir().join(format!("ddn-grant2-{}",hex::encode(nonce)));
        std::fs::create_dir(&directory).unwrap();std::fs::create_dir(directory.join("root")).unwrap();
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:"명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #빨강. }.".into()}]},String::new()).unwrap();
        (directory,project)
    }
    #[test]
    fn vc04_source_grant2_actual_register_cold_consume_and_uncertain_rejection(){
        for mode in ["normal","flush","readback"] {
            let (directory,mut project)=fixture();let root=directory.join("root");
            let artifact=project.save().unwrap();
            let saved=crate::runtime_surface::save_initial_authored_source_native(&artifact,&[1],&root).unwrap();
            let locator=saved.grant_locator.clone().expect("actual Native protected registration locator");
            assert!(crate::runtime_surface::save_initial_authored_source_native(&artifact,&[1],&root).is_err(),"existing source root is never re-provisioned");
            let no_op=crate::runtime_surface::update_authored_source_native(&artifact,&[9],&root,&saved.project_identity,&locator,&saved.revision_identity,&saved.revision_integrity).unwrap();
            assert_eq!(no_op.revision_identity,[1]);assert_eq!(no_op.revision_integrity,saved.revision_integrity);
            let mut latest=saved;
            for (revision,color) in [(2,"파랑"),(3,"노랑")] {
                project.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=
                    format!("명찰:이름씨 := {{ 이름:글. }}. 실행:움직씨 := {{ 색 <- #{color}. }}.");
                let next=crate::runtime_surface::update_authored_source_native(&project.save().unwrap(),&[revision],&root,&latest.project_identity,&locator,&latest.revision_identity,&latest.revision_integrity).unwrap();
                assert_eq!(next.grant_locator.as_deref(),Some(locator.as_str()));
                assert_ne!(next.revision_integrity,latest.revision_integrity);assert_ne!(next.pesi,latest.pesi);latest=next;
            }
            let reread=crate::runtime_surface::read_authored_source_native(&root,&latest.project_identity,&locator).unwrap();
            assert_eq!(reread.revision_integrity,latest.revision_integrity);assert_eq!(reread.revision_identity,[3]);
            assert!(crate::runtime_surface::update_authored_source_native(&artifact,&[12],&root,&latest.project_identity,&locator,&[1],&no_op.revision_integrity).err().expect("stale source rejected").contains("E_SOURCE_SUCCESSOR_STALE"));
            project.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=
                "명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #회색. }.".into();
            for (fault,revision,expected) in [(1,4,"BeforeCommit(WriteFailed)"),(2,5,"AfterCommitUnconfirmed(RereadFailed)")] {
                super::super::super::source_revision::inject_save_fault(fault);
                let failed=crate::runtime_surface::update_authored_source_native(&project.save().unwrap(),&[revision],&root,&latest.project_identity,&locator,&latest.revision_identity,&latest.revision_integrity);
                super::super::super::source_revision::inject_save_fault(0);
                assert!(failed.err().expect("no partial success").contains(expected));
                let confirmed=crate::runtime_surface::read_authored_source_native(&root,&latest.project_identity,&locator).unwrap();
                if fault==1 {assert_eq!(confirmed.revision_identity,latest.revision_identity);assert_eq!(confirmed.revision_integrity,latest.revision_integrity);}
                else {assert_eq!(confirmed.revision_identity,[5]);latest=confirmed;}
            }
            std::fs::write(directory.join("locator.txt"),&locator).unwrap();
            std::fs::write(directory.join("revision.txt"),hex::encode(&latest.revision_identity)).unwrap();
            std::fs::write(directory.join("project.txt"),hex::encode(latest.project_identity)).unwrap();
            std::fs::write(directory.join("integrity.txt"),hex::encode(latest.revision_integrity)).unwrap();
            let child=std::process::Command::new(std::env::current_exe().unwrap()).args([
                "--exact","snapshot_v2_durable_transaction::native::root_grant::windows::source_first_use_tests::vc04_source_grant2_cold_child","--ignored","--nocapture"])
                .env("DDN_SOURCE_GRANT2_DIRECTORY",&directory).env("DDN_SOURCE_GRANT2_FAULT",mode).output().unwrap();
            assert!(child.status.success(),"{mode}: {}\n{}",String::from_utf8_lossy(&child.stdout),String::from_utf8_lossy(&child.stderr));
            let result:serde_json::Value=serde_json::from_slice(&std::fs::read(directory.join("result.json")).unwrap()).unwrap();
            assert_eq!(result["cold_started_rejected"],true);assert_eq!(result["world_published"],false);
            assert_eq!(result["permit"],mode=="normal");
            println!("GRANT2_COLD {mode} {} {}",directory.display(),result);
        }
    }
    #[test]
    #[ignore="exact child of actual protected source Grant2 workflow"]
    fn vc04_source_grant2_cold_child(){
        let directory=std::path::PathBuf::from(std::env::var_os("DDN_SOURCE_GRANT2_DIRECTORY").expect("explicit parent fixture"));
        let mode=std::env::var("DDN_SOURCE_GRANT2_FAULT").unwrap();
        let locator=std::fs::read_to_string(directory.join("locator.txt")).unwrap();
        let project=hex::decode(std::fs::read_to_string(directory.join("project.txt")).unwrap()).unwrap();
        let integrity=std::fs::read_to_string(directory.join("integrity.txt")).unwrap();
        let expected_integrity:[u8;32]=hex::decode(&integrity).unwrap().try_into().unwrap();
        let revision=hex::decode(std::fs::read_to_string(directory.join("revision.txt")).unwrap()).unwrap();
        let mut adapter=super::super::super::SqliteAdapterV1::open_source_revision(&directory.join("root")).unwrap();
        let mut foreign=project.clone();foreign[0]^=1;
        assert!(adapter.confirm_unused_source_root_grant(&locator,&foreign).is_err());
        let mut result=serde_json::json!({"world_published":false,"permit":false});
        if mode=="normal" {
            drop(adapter);
            assert!(crate::runtime_surface::begin_first_authored_source_native(&directory.join("root"),&project,&locator,&[99],&expected_integrity).is_err(),"stale selected source must not consume first use");
            let session=crate::runtime_surface::begin_first_authored_source_native(&directory.join("root"),&project,&locator,&revision,&expected_integrity).unwrap();
            let source=session.source_readback().unwrap();assert_eq!(hex::encode(source.revision_integrity),integrity);
            assert_eq!(source.grant_locator.as_deref(),Some(locator.as_str()));
            result["permit"]=true.into();drop(session);
        }else{
        let grant=adapter.confirm_unused_source_root_grant(&locator,&project).unwrap();
        if mode=="flush"{IO_FAULT.with(|f|f.set(Some((Namespace::Grant,IoStep::Flush))));}
        if mode=="readback"{IO_FAULT.with(|f|f.set(Some((Namespace::Grant,IoStep::FreshRead))));}
        let started=grant.consume_first_source_world(&adapter,&project,&revision,&expected_integrity);
        match started {
            Ok(permit)=>{
                assert_eq!(mode,"normal");permit.verify_before_world(&adapter).unwrap();
                assert_eq!(hex::encode(permit.source().integrity()),integrity);
                result["permit"]=true.into();
                // Deliberately drop before any VM/Attempt. No deletion or
                // registry fault is needed to prove cold cannot retry this.
                drop(permit);
            },
            Err(error)=>{
                assert_ne!(mode,"normal");assert_eq!(error.phase,"consume-unconfirmed");
                assert_eq!(error.cause,if mode=="flush"{Failure::SyncFailed}else{Failure::RereadFailed});
                result["cause"]=format!("{:?}",error.cause).into();
            },
        }
        drop(adapter);
        }
        let mut cold=super::super::super::SqliteAdapterV1::open_source_revision(&directory.join("root")).unwrap();
        let error=cold.confirm_unused_source_root_grant(&locator,&project).err().expect("Started is never Unused");
        assert_eq!(error.cause,Failure::RecoveryRequired);result["cold_started_rejected"]=true.into();
        std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}
