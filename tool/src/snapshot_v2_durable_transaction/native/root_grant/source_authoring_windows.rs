// Exact R1.1 authoring branch. Included in the same Windows protected owner;
// existing Unused/first-use/ordinary-world admission remains unchanged.
use super::{AuthoringHeadV1,AdmittedAuthoringBaseV1,PreparedAuthoringSourceV1,DurablySavedAuthoringSourceV1};
use super::source_transition::{Record as TransitionRecord,Phase as TransitionPhase};
use super::{NewSourceRunPermitV1,RestartSourceWorldAttemptV1};
use super::RestartSourceHistoryV1;

fn authoring_head(adapter:&super::super::SqliteAdapterV1,project:&[u8])->Result<AuthoringHeadV1,Failure>{
    let db=adapter.connect()?;
    db.query_row("SELECT history_revision_identity,revision_integrity_sha256,payload_key,payload_kind FROM vc04_project_heads WHERE project_identity=?1",[project],|r|{
        let revision=r.get_ref(0)?.as_blob()?;let integrity=r.get_ref(1)?.as_blob()?;let key=r.get_ref(2)?.as_blob()?;let kind=r.get::<_,i64>(3)?;
        if project.len()!=32||revision.is_empty()||revision.len()>4096||integrity.len()!=32||key.len()!=32||!(0..=1).contains(&kind){return Err(rusqlite::Error::InvalidQuery);}
        Ok(AuthoringHeadV1{project:project.to_vec(),revision:revision.to_vec(),integrity:integrity.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?,
            key:key.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?,kind:kind as u8})
    }).map_err(super::super::database_error)
}
impl AdmittedAuthoringBaseV1 {
    pub(crate) fn consume_new_run(self,adapter:&super::super::SqliteAdapterV1)->Result<NewSourceRunPermitV1,Failure>{
        self.verify_current(adapter,self.transition.as_ref())?;
        let ready=self.transition.as_ref().ok_or(Failure::CustodyConflict)?;
        if ready.phase!=TransitionPhase::Ready||self.head.kind!=0||self.head.revision!=self.source.revision(){return Err(Failure::CustodyConflict);}
        let mut consumed=ready.clone();consumed.phase=TransitionPhase::Consumed;
        WindowsRootGrantStoreV1::write_transition_record(&self.grant.record,Some(ready),&consumed)?;
        #[cfg(any(test,feature="beta2-crash-probe"))]
        super::super::crash_probe::checkpoint("authoring_after_consume");
        self.verify_current(adapter,Some(&consumed))?;
        Ok(NewSourceRunPermitV1{grant:self.grant,anchor_attempt:self.attempt,consumed,source:self.source})
    }
    fn verify_current(&self,adapter:&super::super::SqliteAdapterV1,expected_record:Option<&TransitionRecord>)->Result<(),Failure>{
        self.grant.verify_root(adapter,&self.head.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=self.attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.grant.record)?.as_ref()!=expected_record
            ||authoring_head(adapter,&self.head.project)?!=self.head{return Err(Failure::CustodyConflict);}
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let checked=super::super::source_world::check_mixed_source_world_history(&db,&self.head.project,self.attempt.epoch,&self.attempt.transaction,
            &self.attempt.request_hash,&self.attempt.token_hash,&self.attempt.record_hash,self.transition.as_ref())?;
        if checked.latest_source_revision!=self.source.revision()||checked.latest_source_integrity!=*self.source.integrity()
            ||checked.latest_source_pesi!=*self.source.pesi(){return Err(Failure::CustodyConflict);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        self.grant.verify_root(adapter,&self.head.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=self.attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.grant.record)?.as_ref()!=expected_record
            ||authoring_head(adapter,&self.head.project)?!=self.head{return Err(Failure::CustodyConflict);}
        Ok(())
    }
    pub(crate) fn head(&self)->&AuthoringHeadV1{&self.head}
    pub(crate) fn source(&self)->&SourceOnly{&self.source}
    pub(crate) fn prepare(self,adapter:&super::super::SqliteAdapterV1,artifact:&str,revision:&[u8])->Result<PreparedAuthoringSourceV1,String>{
        self.verify_current(adapter,self.transition.as_ref()).map_err(|e|format!("E_SOURCE_AUTHORING_BASE: {e:?}"))?;
        let source=crate::authored_project::prepare_authoring_source(&self.source,artifact,&self.head.revision,revision)?;
        let ordinal=self.transition.as_ref().map_or(Some(1),|t|t.ordinal.checked_add(1)).ok_or("E_SOURCE_AUTHORING_ORDINAL")?;
        let anchor=&self.history.last_world;
        let prepared=TransitionRecord{nonce:self.grant.record.base.nonce,generation:self.grant.record.base.generation,ordinal,phase:TransitionPhase::Prepared,
            anchor_digest:anchor.digest,anchor_revision:anchor.binding.new_revision.clone(),anchor_integrity:anchor.binding.revision_integrity,
            anchor_epoch:self.attempt.epoch,anchor_record:self.attempt.record_hash,parent_kind:self.head.kind,parent_revision:self.head.revision.clone(),parent_integrity:self.head.integrity,
            source_revision:source.revision().to_vec(),source_integrity:*source.integrity(),source_pesi:*source.pesi(),bound_digest:[0;32],first_bound_epoch:0};
        self.verify_current(adapter,self.transition.as_ref()).map_err(|e|format!("E_SOURCE_AUTHORING_BASE: {e:?}"))?;
        WindowsRootGrantStoreV1::write_transition_record(&self.grant.record,self.transition.as_ref(),&prepared).map_err(|e|format!("E_SOURCE_AUTHORING_PREPARED: {e:?}"))?;
        self.verify_current(adapter,Some(&prepared)).map_err(|e|format!("E_SOURCE_AUTHORING_PREPARED_REREAD: {e:?}"))?;
        Ok(PreparedAuthoringSourceV1{base:self,source,prepared})
    }
}
impl NewSourceRunPermitV1 {
    pub(crate) fn source(&self)->&SourceOnly{&self.source}
    pub(crate) fn locator(&self)->String{self.grant.locator()}
    pub(crate) fn verify_before_world(&self,adapter:&super::super::SqliteAdapterV1)->Result<(),Failure>{
        self.grant.verify_root(adapter,self.source.project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=self.anchor_attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.grant.record)?.as_ref()!=Some(&self.consumed){return Err(Failure::CustodyConflict);}
        let mut ready=self.consumed.clone();ready.phase=TransitionPhase::Ready;
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let checked=super::super::source_world::check_mixed_source_world_history(&db,self.source.project(),self.anchor_attempt.epoch,&self.anchor_attempt.transaction,
            &self.anchor_attempt.request_hash,&self.anchor_attempt.token_hash,&self.anchor_attempt.record_hash,Some(&ready))?;
        if checked.current_world.is_some()||checked.latest_source_revision!=self.source.revision()||checked.latest_source_integrity!=*self.source.integrity(){return Err(Failure::CustodyConflict);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        self.grant.verify_root(adapter,self.source.project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.grant.record.base)?!=self.anchor_attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.grant.record)?.as_ref()!=Some(&self.consumed){return Err(Failure::CustodyConflict);}Ok(())
    }
    fn check_request(&self,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        request:&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<(),Failure>{
        request.check_source(context)?;let b=request.binding();
        if b.project!=self.source.project()||b.expected_parent!=self.source.revision()||b.execution_profile!=self.source.profile()||b.pesi!=*self.source.pesi()
            ||crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(context.owned()).map_err(|_|Failure::CustodyConflict)?.as_bytes()!=self.source.payload(){return Err(Failure::CustodyConflict);}
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&request.payload().snapshot,context.pesi().as_raw())?;
        let descriptor=crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(layout.supplies[9],context.owned())?;
        if !descriptor.prior_durable_receipts.is_empty(){return Err(Failure::CustodyConflict);}Ok(())
    }
    pub(crate) fn check_actual_world_request(&self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,request:&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<(),Failure>{
        self.verify_before_world(adapter)?;self.check_request(context,request)
    }
    pub(crate) fn start_world(self,adapter:&super::super::SqliteAdapterV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)->Result<RestartSourceWorldAttemptV1,Failure>{
        use sha2::{Digest,Sha256};self.check_actual_world_request(adapter,context,&request)?;
        let digest=request.payload().digest;let mut token=vec![1];token.extend(digest);
        let pending=AttemptRecord{nonce:self.grant.record.base.nonce,generation:self.grant.record.base.generation,
            epoch:self.anchor_attempt.epoch.checked_add(1).ok_or(Failure::LimitExceeded)?,transaction:digest,request_hash:Sha256::digest(request.raw()).into(),
            token_hash:Sha256::digest(token).into(),mode:AttemptMode::Pending,record_hash:[0;32]};
        WindowsRootGrantStoreV1::write_next_source_pending(&self.grant.record,&self.anchor_attempt,&pending)?;
        let start=RestartSourceWorldAttemptV1{permit:self,attempt:pending,request};start.check_current(adapter,context)?;Ok(start)
    }
}
impl RestartSourceWorldAttemptV1 {
    pub(crate) fn request(&self)->&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1{&self.request}
    pub(crate) fn source(&self)->&SourceOnly{&self.permit.source}
    pub(crate) fn check_current(&self,adapter:&super::super::SqliteAdapterV1,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{
        self.permit.grant.verify_root(adapter,self.source().project())?;self.permit.check_request(context,&self.request)?;
        if WindowsRootGrantStoreV1::read_attempt(&self.permit.grant.record.base)?!=self.attempt||self.attempt.mode!=AttemptMode::Pending
            ||WindowsRootGrantStoreV1::read_transition_record(&self.permit.grant.record)?.as_ref()!=Some(&self.permit.consumed){return Err(Failure::CustodyConflict);}
        let actual=super::super::source_chain::read(&adapter.connect()?,self.source().project(),self.source().revision())?;
        if actual.artifact().map_err(|_|Failure::CustodyConflict)?!=self.source().artifact().map_err(|_|Failure::CustodyConflict)?{return Err(Failure::RereadMismatch);}Ok(())
    }
    pub(crate) fn retain_source_witness(&self,adapter:&super::super::SqliteAdapterV1,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        sql:&super::super::source_world::SourceSqlRetainedV2)->Result<RetainedAttemptWitnessV1,Failure>{
        use sha2::{Digest,Sha256};
        if !sql.start().restart_is(self){return Err(Failure::CustodyConflict);}
        self.check_current(adapter,context)?;sql.revalidate(adapter)?;
        let mut witness=self.attempt.clone();witness.mode=AttemptMode::RecordWitness;witness.record_hash=Sha256::digest(sql.record()).into();witness.validate()?;
        let mut bytes=witness.encode()?;let result=WindowsRootGrantStoreV1::write_plaintext(Namespace::Attempt,&self.permit.grant.record.base.nonce,&bytes,false);bytes.fill(0);result?;
        self.permit.grant.verify_root(adapter,self.source().project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.permit.grant.record.base)?!=witness{return Err(Failure::RereadMismatch);}sql.revalidate(adapter)?;
        if sql.outcome().failure().is_none(){
            let mut bound=self.permit.consumed.clone();bound.phase=TransitionPhase::Bound;bound.bound_digest=self.request.payload().digest;bound.first_bound_epoch=witness.epoch;
            let check=||->Result<(),Failure>{
                self.permit.grant.verify_root(adapter,self.source().project())?;
                if WindowsRootGrantStoreV1::read_attempt(&self.permit.grant.record.base)?!=witness{return Err(Failure::CustodyConflict);}
                let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
                let checked=super::super::source_world::check_mixed_source_world_history(&db,self.source().project(),witness.epoch,&witness.transaction,
                    &witness.request_hash,&witness.token_hash,&witness.record_hash,Some(&bound))?;
                if checked.current_world.is_none(){return Err(Failure::CustodyConflict);}
                db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;Ok(())
            };
            check()?;WindowsRootGrantStoreV1::write_transition_record(&self.permit.grant.record,Some(&self.permit.consumed),&bound)?;check()?;
            if WindowsRootGrantStoreV1::read_transition_record(&self.permit.grant.record)?.as_ref()!=Some(&bound){return Err(Failure::RereadMismatch);}
        }
        Ok(RetainedAttemptWitnessV1{_attempt:witness})
    }
}
impl PreparedAuthoringSourceV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) fn source(&self)->&SourceOnly{&self.source}
    pub(in crate::snapshot_v2_durable_transaction::native) fn parent(&self)->&AuthoringHeadV1{&self.base.head}
    pub(in crate::snapshot_v2_durable_transaction::native) fn verify_before_save(&self,adapter:&super::super::SqliteAdapterV1)->Result<(),Failure>{
        self.base.verify_current(adapter,Some(&self.prepared))
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn confirm_saved(self,adapter:&super::super::SqliteAdapterV1)->Result<DurablySavedAuthoringSourceV1,Failure>{
        #[cfg(any(test,feature="beta2-crash-probe"))]
        super::super::crash_probe::checkpoint("authoring_after_source_cas");
        let mut ready=self.prepared.clone();ready.phase=TransitionPhase::Ready;
        self.base.grant.verify_root(adapter,self.source.project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.base.grant.record.base)?!=self.base.attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.base.grant.record)?.as_ref()!=Some(&self.prepared){return Err(Failure::CustodyConflict);}
        adapter.confirm_container_durable()?;
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let read=super::super::source_chain::read(&db,self.source.project(),self.source.revision())?;
        if read.artifact().map_err(|_|Failure::CustodyConflict)?!=self.source.artifact().map_err(|_|Failure::CustodyConflict)?{return Err(Failure::RereadMismatch);}
        let checked=super::super::source_world::check_mixed_source_world_history(&db,self.source.project(),self.base.attempt.epoch,&self.base.attempt.transaction,
            &self.base.attempt.request_hash,&self.base.attempt.token_hash,&self.base.attempt.record_hash,Some(&ready))?;
        if checked.current_world.is_some()||checked.latest_source_revision!=self.source.revision(){return Err(Failure::CustodyConflict);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        WindowsRootGrantStoreV1::write_transition_record(&self.base.grant.record,Some(&self.prepared),&ready)?;
        self.base.grant.verify_root(adapter,self.source.project())?;
        if WindowsRootGrantStoreV1::read_attempt(&self.base.grant.record.base)?!=self.base.attempt
            ||WindowsRootGrantStoreV1::read_transition_record(&self.base.grant.record)?.as_ref()!=Some(&ready){return Err(Failure::RereadMismatch);}
        Ok(DurablySavedAuthoringSourceV1{grant:self.base.grant,attempt:self.base.attempt,ready,source:self.source})
    }
}
impl DurablySavedAuthoringSourceV1 {
    pub(crate) fn source(&self)->&SourceOnly{&self.source}
    pub(crate) fn locator(&self)->String{self.grant.locator()}
}

fn checked_restart_history(adapter:&super::super::SqliteAdapterV1,grant:&AdmittedSourceGrantV2,attempt:&AttemptRecord,record:&TransitionRecord,
    context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>)
    ->Result<(super::super::source_world::CheckedMixedRevisionHistory,SourceOnly),Failure>{
    if record.phase!=TransitionPhase::Bound||attempt.mode!=AttemptMode::RecordWitness{return Err(Failure::RecoveryRequired);}
    let project=hex::decode(context.owned().project().authored_identity.logical_dependency_id.as_str()).map_err(|_|Failure::CustodyConflict)?;
    grant.verify_root(adapter,&project)?;
    if WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?!=*attempt||WindowsRootGrantStoreV1::read_transition_record(&grant.record)?.as_ref()!=Some(record){return Err(Failure::CustodyConflict);}
    adapter.confirm_container_durable()?;
    let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
    let checked=super::super::source_world::check_mixed_source_world_history(&db,&project,attempt.epoch,&attempt.transaction,&attempt.request_hash,&attempt.token_hash,&attempt.record_hash,Some(record))?;
    let current=checked.current_world.as_ref().ok_or(Failure::CustodyConflict)?;
    let source=super::super::source_chain::read(&db,&project,&checked.latest_source_revision)?;
    let initial=super::super::source_chain::read(&db,&project,&checked.initial_source_revision)?;
    let FirstUse::Started(first)=&grant.record.state else{return Err(Failure::CustodyConflict);};
    if current.envelope!=context.revision_envelope()||current.binding.pesi!=*context.pesi().as_raw()
        ||grant.record.initial.revision!=checked.initial_source_revision||grant.record.initial.integrity!=checked.initial_source_integrity||grant.record.base.registration_pesi!=*initial.pesi()
        ||first.revision!=checked.first_runtime_source_revision||first.integrity!=checked.first_runtime_source_integrity
        ||crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(context.owned()).map_err(|_|Failure::CustodyConflict)?.as_bytes()!=source.payload(){return Err(Failure::CustodyConflict);}
    db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
    grant.verify_root(adapter,&project)?;
    if WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?!=*attempt||WindowsRootGrantStoreV1::read_transition_record(&grant.record)?.as_ref()!=Some(record){return Err(Failure::CustodyConflict);}
    Ok((checked,source))
}
impl ReadmittedFirstSourceWorldV2 {
    fn restart_historical_payload(&self,adapter:&super::super::SqliteAdapterV1,index:usize,context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<super::super::StoredPayloadV1,Failure>{
        let restart=self.restart.as_ref().ok_or(Failure::CustodyConflict)?;
        let (checked,_)=checked_restart_history(adapter,&self._grant,&self._attempt,&restart.record,&context.source_context())?;
        if checked.current_world.as_ref()!=Some(&self.payload)||checked.current_segment_digests!=self.history{return Err(Failure::CustodyConflict);}
        let digest=self.history.get(index).ok_or(Failure::InvalidInput)?;
        let payload=super::super::source_world::read_source_history_payload(&adapter.connect()?,digest)?;
        self._grant.verify_root(adapter,&self.payload.binding.project)?;
        if WindowsRootGrantStoreV1::read_attempt(&self._grant.record.base)?!=self._attempt{return Err(Failure::CustodyConflict);}Ok(payload)
    }
    fn start_restart_next_world(self,adapter:&super::super::SqliteAdapterV1,previous:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        next:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,request:crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1)
        ->Result<NextSourceWorldAttemptStartV1,Failure>{
        use crate::unit_wire::resumable_complete_state as complete;
        let restart=self.restart.as_ref().ok_or(Failure::CustodyConflict)?;
        let (checked,_)=checked_restart_history(adapter,&self._grant,&self._attempt,&restart.record,&previous.source_context())?;
        if checked.current_world.as_ref()!=Some(&self.payload)||checked.current_segment_digests!=self.history{return Err(Failure::CustodyConflict);}
        request.check_source(next)?;let b=request.binding();
        if b.project!=self.payload.binding.project||b.expected_parent!=self.payload.binding.new_revision||b.new_revision==self.payload.binding.new_revision
            ||b.pesi!=self.payload.binding.pesi||b.execution_profile!=self.payload.binding.execution_profile
            ||next.owned().project().save().map_err(|_|Failure::CustodyConflict)?!=previous.owned().project().save().map_err(|_|Failure::CustodyConflict)?{return Err(Failure::CustodyConflict);}
        let old_layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&self.payload.snapshot,previous.pesi().as_raw())?;
        let old_descriptor=complete::rebuild_source_descriptor_candidate(old_layout.supplies[9],previous.owned())?;
        let new_layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&request.payload().snapshot,next.pesi().as_raw())?;
        let new_descriptor=complete::rebuild_source_descriptor_candidate(new_layout.supplies[9],next.owned())?;
        let receipt=crate::snapshot_v2_host_bridge::encode_committed_receipt_candidate(&self.payload.binding,adapter.capability()?.as_bytes())?;
        let n=old_descriptor.prior_durable_receipts.len();
        if new_descriptor.source_descriptor_prefix!=old_descriptor.source_descriptor_prefix||new_descriptor.execution_instance!=old_descriptor.execution_instance
            ||new_descriptor.successful_madi_count<old_descriptor.successful_madi_count||new_descriptor.prior_durable_receipts.len()!=n.checked_add(1).ok_or(Failure::LimitExceeded)?
            ||new_descriptor.prior_durable_receipts[..n]!=old_descriptor.prior_durable_receipts||new_descriptor.prior_durable_receipts.last().copied()!=Some(receipt.as_slice()){return Err(Failure::CustodyConflict);}
        let Self{_grant:grant,_attempt:old,payload,history:_,restart}=self;
        let restart=restart.ok_or(Failure::CustodyConflict)?;
        let digest=request.payload().digest;let mut token=vec![1];token.extend(digest);
        let mut pending=AttemptRecord::initial_pending(&grant.record.base,digest,request.raw(),&token)?;pending.epoch=old.epoch.checked_add(1).ok_or(Failure::LimitExceeded)?;
        WindowsRootGrantStoreV1::write_next_source_pending(&grant.record,&old,&pending)?;
        let previous=super::super::StoredHeadV1{project:payload.binding.project,revision:payload.binding.new_revision,integrity:payload.binding.revision_integrity,digest:payload.digest};
        let start=NextSourceWorldAttemptStartV1{grant,source:restart.source,attempt:pending,request,previous,restart:Some(restart.record)};start.check_current(adapter,next)?;Ok(start)
    }
}
impl super::super::SqliteAdapterV1 {
    #[cfg(test)]
    pub(crate) fn invalidate_source_transition_for_test(&self,locator:&str)->Result<(),Failure>{
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        WindowsRootGrantStoreV1::read_source_grant(&nonce)?;
        WindowsRootGrantStoreV1::write_plaintext(Namespace::SourceTransition,&nonce,b"invalid-test-only-transition",false)
    }
    pub(crate) fn current_authoring_source_if_ready(&mut self,locator:&str,project:&[u8])->Result<Option<AdmittedAuthoringBaseV1>,Failure>{
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&nonce)?;self.pin_objects_for_grant(false)?;
        let grant=AdmittedSourceGrantV2{record,pinned:self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone(),lease:self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone()};
        grant.verify_root(self,project)?;
        if matches!(grant.record.state,FirstUse::Unused){return Ok(None);}
        let Some(transition)=WindowsRootGrantStoreV1::read_transition_record(&grant.record)? else{return Ok(None);};
        match transition.phase{TransitionPhase::Bound=>return Ok(None),TransitionPhase::Ready=>{},_=>return Err(Failure::RecoveryRequired)}
        let head=authoring_head(self,project)?;
        if head.kind!=0{return Err(Failure::RecoveryRequired);}
        let candidate=super::super::source_chain::read(&self.connect()?,project,&head.revision)?;
        let artifact=candidate.owned().project().save().map_err(|_|Failure::CustodyConflict)?;
        self.admit_authoring_base(locator,&head,&artifact).map(Some)
    }
    fn readmit_restart_source_world(&mut self,grant:AdmittedSourceGrantV2,attempt:AttemptRecord,record:TransitionRecord,
        context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>)->Result<ReadmittedFirstSourceWorldV2,Failure>{
        let (checked,source)=checked_restart_history(self,&grant,&attempt,&record,context)?;
        let payload=checked.current_world.ok_or(Failure::CustodyConflict)?;
        Ok(ReadmittedFirstSourceWorldV2{_grant:grant,_attempt:attempt,payload,history:checked.current_segment_digests,restart:Some(RestartSourceHistoryV1{source,record})})
    }
    pub(crate) fn selected_authoring_head(&self,project:&[u8])->Result<AuthoringHeadV1,Failure>{authoring_head(self,project)}
    pub(crate) fn admit_authoring_base(&mut self,locator:&str,expected:&AuthoringHeadV1,original_artifact:&str)->Result<AdmittedAuthoringBaseV1,Failure>{
        if self.schema_profile!=super::super::NativeSchemaProfile::SourceRevision||locator.len()!=64{return Err(Failure::InvalidInput);}
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        let record=WindowsRootGrantStoreV1::read_source_grant(&nonce)?;
        if !matches!(record.state,FirstUse::Started(_)){return Err(Failure::CustodyConflict);}
        self.pin_objects_for_grant(false)?;
        let grant=AdmittedSourceGrantV2{record,pinned:self.pinned_root.as_ref().ok_or(Failure::UnsupportedDurability)?.clone(),lease:self.root_lease.as_ref().ok_or(Failure::UnsupportedDurability)?.clone()};
        grant.verify_root(self,&expected.project)?;
        let attempt=WindowsRootGrantStoreV1::read_attempt(&grant.record.base)?;
        if attempt.mode!=AttemptMode::RecordWitness{return Err(Failure::RecoveryRequired);}
        let transition=WindowsRootGrantStoreV1::read_transition_record(&grant.record)?;
        if transition.as_ref().is_some_and(|t|!matches!(t.phase,TransitionPhase::Ready|TransitionPhase::Bound)){return Err(Failure::RecoveryRequired);}
        self.confirm_container_durable()?;
        if authoring_head(self,&expected.project)?!=*expected{return Err(Failure::StaleRevision);}
        let db=self.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let history=super::super::source_world::check_mixed_source_world_history(&db,&expected.project,attempt.epoch,&attempt.transaction,&attempt.request_hash,&attempt.token_hash,&attempt.record_hash,transition.as_ref())?;
        let source=super::super::source_chain::read(&db,&expected.project,&history.latest_source_revision)?;
        let initial=super::super::source_chain::read(&db,&expected.project,&history.initial_source_revision)?;
        let FirstUse::Started(first)=&grant.record.state else{return Err(Failure::CustodyConflict);};
        if grant.record.initial.revision!=history.initial_source_revision||grant.record.initial.integrity!=history.initial_source_integrity
            ||grant.record.base.registration_pesi!=*initial.pesi()||first.revision!=history.first_runtime_source_revision||first.integrity!=history.first_runtime_source_integrity
            ||source.owned().project().save().map_err(|_|Failure::CustodyConflict)?!=original_artifact{return Err(Failure::CustodyConflict);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;drop(db);
        let base=AdmittedAuthoringBaseV1{grant,attempt,transition,head:expected.clone(),source,history};
        base.verify_current(self,base.transition.as_ref())?;Ok(base)
    }
}
