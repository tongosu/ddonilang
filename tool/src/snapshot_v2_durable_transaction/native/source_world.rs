//! R5 source-parent first W. Separate typed entry; old INITIAL stays empty.
//! Prepared persistence never advances the source head or publishes success.
use super::{SqliteAdapterV1,NativeSchemaProfile,Failure,SourceWorldAttemptStartV2,StoredPayloadV1,TransactionBindingV1};
use crate::snapshot_v2_durable_transaction::recovery_custody::{InitialCustodyRowV1,encode_prepared_candidate};
use crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment;
use super::NextSourceWorldAttemptStartV1;
use rusqlite::{params,OptionalExtension};
use crate::snapshot_v2_durable_transaction as owner;
use owner::{TransactionEvidenceV1,NativeOwnerOutcomeV1,NativeOwnerOutcomeKindV1,NativeTransactionFailureV1,SnapshotV2TransactionStateV1 as State};
use owner::recovery_custody::{encode_verified_candidate,encode_actual_outcome_candidate};
#[cfg(test)]
std::thread_local!{static SOURCE_WORLD_FAULT:std::cell::Cell<u8>=const{std::cell::Cell::new(0)};}
#[cfg(test)]
pub(crate) fn inject_fault(phase:u8){assert!(phase<=3);SOURCE_WORLD_FAULT.with(|v|v.set(phase));}

/// Closed, actual-owner variants. Neither can be made from a JSON status or
/// an admitted frame alone. First-S and subsequent-W predecessor guards differ.
pub(in crate::snapshot_v2_durable_transaction) enum SourceWorldStart {
    First(SourceWorldAttemptStartV2),Next(NextSourceWorldAttemptStartV1),
}
impl SourceWorldStart {
    pub(in crate::snapshot_v2_durable_transaction) fn first_is(&self,value:&SourceWorldAttemptStartV2)->bool{matches!(self,Self::First(first) if std::ptr::eq(first,value))}
    pub(in crate::snapshot_v2_durable_transaction) fn next_is(&self,value:&NextSourceWorldAttemptStartV1)->bool{matches!(self,Self::Next(next) if std::ptr::eq(next,value))}
    fn is_first(&self)->bool{matches!(self,Self::First(_))}
    fn request(&self)->&crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1{match self{Self::First(s)=>s.request(),Self::Next(s)=>s.request()}}
    fn source(&self)->&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision{match self{Self::First(s)=>s.source(),Self::Next(s)=>s.source()}}
    fn expected(&self)->super::StoredHeadV1{match self{
        Self::First(s)=>super::StoredHeadV1{project:s.source().project().to_vec(),revision:s.source().revision().to_vec(),integrity:*s.source().integrity(),digest:*s.source().integrity()},
        Self::Next(s)=>s.previous().clone(),
    }}
    fn check_current(&self,adapter:&SqliteAdapterV1,context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{match self{Self::First(s)=>s.check_current(adapter,context),Self::Next(s)=>s.check_current(adapter,context)}}
    fn retain_source_witness(&self,adapter:&SqliteAdapterV1,context:&SourceBoundSavedRuntimeAttachment<'_>,sql:&SourceSqlRetainedV2)->Result<super::RetainedAttemptWitnessV1,Failure>{
        match self{Self::First(s)=>s.retain_source_witness(adapter,context,sql),Self::Next(s)=>s.retain_source_witness(adapter,context,sql)}
    }
}

pub(crate) struct RetainedFirstSourceWorldV2 {
    sql:SourceSqlRetainedV2,
    _witness:super::RetainedAttemptWitnessV1,
}
impl RetainedFirstSourceWorldV2 {
    pub(crate) fn outcome(&self)->&NativeOwnerOutcomeV1{&self.sql.outcome}
}
/// Only the actual continuation below constructs this, after exact full SQL
/// prefix readback. Not a record decoder, status BOOL, or receipt.
pub(in crate::snapshot_v2_durable_transaction) struct SourceSqlRetainedV2 {
    prepared:PreparedFirstSourceWorldV2,
    outcome:NativeOwnerOutcomeV1,
    verified:Option<(Vec<u8>,[u8;32])>,
    terminal:Vec<u8>,terminal_hash:[u8;32],ordinal:u8,
}
impl SourceSqlRetainedV2 {
    pub(in crate::snapshot_v2_durable_transaction) fn start(&self)->&SourceWorldStart{&self.prepared.start}
    pub(in crate::snapshot_v2_durable_transaction) fn record(&self)->&[u8]{&self.terminal}
    pub(in crate::snapshot_v2_durable_transaction) fn revalidate(&self,adapter:&SqliteAdapterV1)->Result<(),Failure>{
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        check_prefix(&db,&self.prepared,self.verified.as_ref(),true)?;
        let previous=self.verified.as_ref().map(|(_,h)|h).unwrap_or(&self.prepared.hash);
        let request=self.start().request();
        if !super::custody_row_matches(&db,&request.payload().digest,self.ordinal,previous,request.raw(),&self.terminal,&self.terminal_hash)?
            ||encode_actual_outcome_candidate(&self.outcome,&self.prepared.record)?!=self.terminal{
            return Err(Failure::RereadMismatch);
        }
        if self.outcome.evidence().state()==State::Committed {
            owner::checked_native_committed_payload(request.payload(),&read_chain(&db,request.payload())?)?;
        }
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;Ok(())
    }
}

pub(crate) struct PreparedFirstSourceWorldV2 {
    pub(super) start:SourceWorldStart,
    pub(super) record:Vec<u8>,
    pub(super) hash:[u8;32],
}
impl PreparedFirstSourceWorldV2 {
    pub(crate) fn request(&self)->&owner::ActualPreparedRequestV1{self.start.request()}
    pub(crate) fn commit(self,adapter:&SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<RetainedFirstSourceWorldV2,String>{
        // Continue from this actual durable Prepared owner. No phase is
        // inferred from a persisted state tag or a caller-provided receipt.
        let request=self.start.request();let payload=request.payload();
        let mut evidence=TransactionEvidenceV1::prepared(payload.binding.clone()).map_err(|e|format!("E_SOURCE_WORLD_PREPARED: {e:?}"))?;
        let continuation=SourceContinuationV2{adapter,source,prepared:&self,issued:Default::default(),committed_once:std::cell::Cell::new(false)};
        let result=(||->Result<(),Failure>{
            if adapter.capability()?.as_bytes()!=b"vc04.sqlite-durability.r1"{return Err(Failure::UnsupportedDurability);}
            self.check_current(adapter,source)?;evidence.synced()?;
            let expected=self.start.expected();
            owner::verify_commit_native_from_synced(&continuation,payload,Some(&expected),&mut evidence,
                |actual|continuation.queue_verified(actual))
        })();
        let verified=continuation.issued.into_inner();
        let outcome=match result {
            Ok(())=>NativeOwnerOutcomeV1{inner:NativeOwnerOutcomeKindV1::Committed(evidence)},
            Err(cause)=>{
                evidence.container_confirmed=false;
                evidence.state=if evidence.head_changed||cause==Failure::RecoveryRequired{State::RecoveryRequired}else{State::Aborted};
                let failure=NativeTransactionFailureV1{state:Some(evidence.state),cause};
                NativeOwnerOutcomeV1{inner:NativeOwnerOutcomeKindV1::Failed{evidence,failure}}
            },
        };
        self.start.check_current(adapter,source).map_err(|e|format!("E_SOURCE_WORLD_RETENTION_CURRENT: outcome={:?}, cause={e:?}",outcome.failure()))?;
        let terminal=encode_actual_outcome_candidate(&outcome,&self.record).map_err(|e|format!("E_SOURCE_WORLD_OUTCOME: {e:?}"))?;
        let ordinal=if verified.is_some(){2}else{1};
        let previous=verified.as_ref().map(|(_,h)|h).unwrap_or(&self.hash);
        let terminal_hash=InitialCustodyRowV1::row_hash(&payload.digest,ordinal,previous,request.raw(),&terminal).map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_HASH: {e:?}"))?;
        let db=adapter.connect().map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_OPEN: {e:?}"))?;
        db.execute_batch("BEGIN IMMEDIATE").map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_BEGIN: outcome={:?}, retention={e}",outcome.failure()))?;
        check_prefix(&db,&self,verified.as_ref(),false).map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_PREFIX: outcome={:?}, retention={e:?}",outcome.failure()))?;
        // A later custody write failure does not replace the actual owner's
        // original state/cause. This diagnostic is not a retained Witness.
        #[cfg(test)]
        if SOURCE_WORLD_FAULT.with(|v|v.get())==3 {
            return Err(format!("E_SOURCE_WORLD_OUTCOME_WRITE: outcome={:?}, retention={:?}",outcome.failure(),Failure::WriteFailed));
        }
        db.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,?2,1,?3,?4,?5,?6)",
            params![payload.digest.as_slice(),ordinal,previous.as_slice(),request.raw(),terminal,terminal_hash.as_slice()])
            .map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_WRITE: outcome={:?}, retention={e}",outcome.failure()))?;
        db.execute_batch("COMMIT").map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_UNCONFIRMED: outcome={:?}, retention={e}",outcome.failure()))?;drop(db);
        adapter.confirm_container_durable().map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_UNCONFIRMED: outcome={:?}, retention={e:?}",outcome.failure()))?;
        let sql=SourceSqlRetainedV2{prepared:self,outcome,verified,terminal,terminal_hash,ordinal};
        sql.revalidate(adapter).map_err(|e|format!("E_SOURCE_WORLD_OUTCOME_READBACK: outcome={:?}, retention={e:?}",sql.outcome.failure()))?;
        let witness=sql.start().retain_source_witness(adapter,source,&sql).map_err(|e|format!("E_SOURCE_WORLD_WITNESS: outcome={:?}, retention={e:?}",sql.outcome.failure()))?;
        Ok(RetainedFirstSourceWorldV2{sql,_witness:witness})
    }
    pub(crate) fn check_current(&self,adapter:&SqliteAdapterV1,source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{
        self.start.check_current(adapter,source)?;
        let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let request=self.start.request();
        if read_world(&db,&request.payload().digest)?.as_ref()!=Some(request.payload())
            ||!super::custody_count_matches(&db,&request.payload().digest,1)?
            ||!super::custody_row_matches(&db,&request.payload().digest,0,&[0;32],request.raw(),&self.record,&self.hash)?{
            return Err(Failure::RereadMismatch);
        }
        check_source_head(&db,&self.start)?;
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;Ok(())
    }
}

/// Sealed nine-method mechanics for the already-durable source-parent entry.
/// It uses the same common verification/CAS/confirmation loop as old INITIAL.
struct SourceContinuationV2<'a,'s>{
    adapter:&'a SqliteAdapterV1,source:&'a SourceBoundSavedRuntimeAttachment<'s>,prepared:&'a PreparedFirstSourceWorldV2,
    issued:std::cell::RefCell<Option<(Vec<u8>,[u8;32])>>,committed_once:std::cell::Cell<bool>,
}
impl SourceContinuationV2<'_,'_>{
    fn queue_verified(&self,evidence:&TransactionEvidenceV1)->Result<(),Failure>{
        let mut issued=self.issued.try_borrow_mut().map_err(|_|Failure::CustodyConflict)?;
        if issued.is_some()||evidence.state()!=State::Verified||evidence.binding!=*self.prepared.start.request().binding()
            ||evidence.head_changed||evidence.container_confirmed{return Err(Failure::CustodyConflict);}
        self.prepared.start.check_current(self.adapter,self.source)?;
        let record=encode_verified_candidate(&self.prepared.record)?;
        let hash=InitialCustodyRowV1::row_hash(&self.prepared.start.request().payload().digest,1,&self.prepared.hash,self.prepared.start.request().raw(),&record)?;
        *issued=Some((record,hash));Ok(())
    }
}
impl owner::native_adapter_sealed::Sealed for SourceContinuationV2<'_,'_>{}
impl owner::SnapshotV2DurabilityAdapterV1 for SourceContinuationV2<'_,'_>{
    type PreparedHandle=super::PreparedWriteV1;
    fn capability(&self)->Result<&'static str,Failure>{self.prepared.start.check_current(self.adapter,self.source)?;self.adapter.capability()}
    fn write_prepared(&self,_:&StoredPayloadV1)->Result<Self::PreparedHandle,Failure>{Err(Failure::UnsupportedDurability)}
    fn sync_prepared(&self,_:&mut Self::PreparedHandle)->Result<(),Failure>{Err(Failure::UnsupportedDurability)}
    fn abort(&self,_:&mut Self::PreparedHandle)->Result<(),Failure>{Err(Failure::UnsupportedDurability)}
    fn reread_prepared(&self,digest:&[u8;32])->Result<Option<StoredPayloadV1>,Failure>{
        if digest!=&self.prepared.start.request().payload().digest{return Err(Failure::CustodyConflict);}
        self.prepared.check_current(self.adapter,self.source)?;read_world(&self.adapter.connect()?,digest)
    }
    fn commit_verified(&self,payload:&StoredPayloadV1,expected:Option<&super::StoredHeadV1>)->Result<(),Failure>{
        let head=self.prepared.start.expected();
        if self.committed_once.replace(true)||payload!=self.prepared.start.request().payload()||expected!=Some(&head){return Err(Failure::CustodyConflict);}
        let issued=self.issued.try_borrow().map_err(|_|Failure::CustodyConflict)?;
        let verified=issued.as_ref().ok_or(Failure::CustodyConflict)?;
        append_verified(self.adapter,self.prepared,verified,self.source)?;
        self.prepared.start.check_current(self.adapter,self.source)?;
        commit_source_head(self.adapter,self.prepared,verified)
    }
    fn confirm_container_durable(&self)->Result<(),Failure>{
        self.prepared.start.check_current(self.adapter,self.source)?;
        #[cfg(test)]
        if SOURCE_WORLD_FAULT.with(|v|v.get())>=2{return Err(Failure::ContainerDurabilityFailed);}
        self.adapter.confirm_container_durable()
    }
    fn reread_committed(&self,digest:&[u8;32],project:&[u8],revision:&[u8])->Result<super::RawChainV1,Failure>{
        let p=self.prepared.start.request().payload();
        if digest!=&p.digest||project!=p.binding.project||revision!=p.binding.new_revision{return Err(Failure::CustodyConflict);}
        self.prepared.start.check_current(self.adapter,self.source)?;
        let db=self.adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
        let issued=self.issued.try_borrow().map_err(|_|Failure::CustodyConflict)?;
        check_prefix(&db,self.prepared,issued.as_ref(),false)?;
        let chain=read_chain(&db,p)?;db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;Ok(chain)
    }
    fn inspect_recovery(&self,digest:&[u8;32],project:&[u8],revision:&[u8])->Result<super::RawChainV1,Failure>{
        self.reread_committed(digest,project,revision)
    }
}
fn check_prefix(db:&rusqlite::Connection,prepared:&PreparedFirstSourceWorldV2,verified:Option<&(Vec<u8>,[u8;32])>,terminal:bool)->Result<(),Failure>{
    let request=prepared.start.request();let digest=&request.payload().digest;
    let count=1+i64::from(verified.is_some())+i64::from(terminal);
    if read_world(db,digest)?.as_ref()!=Some(request.payload())||!super::custody_count_matches(db,digest,count)?
        ||!super::custody_row_matches(db,digest,0,&[0;32],request.raw(),&prepared.record,&prepared.hash)?{
        return Err(Failure::RereadMismatch);
    }
    if let Some((record,hash))=verified {
        if record!=&encode_verified_candidate(&prepared.record)?||!super::custody_row_matches(db,digest,1,&prepared.hash,request.raw(),record,hash)?{
            return Err(Failure::RereadMismatch);
        }
    }Ok(())
}
fn append_verified(adapter:&SqliteAdapterV1,prepared:&PreparedFirstSourceWorldV2,verified:&(Vec<u8>,[u8;32]),source:&SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure>{
    prepared.start.check_current(adapter,source)?;
    let db=adapter.connect()?;db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::WriteFailed)?;
    check_prefix(&db,prepared,None,false)?;check_source_head(&db,&prepared.start)?;
    let request=prepared.start.request();
    db.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,1,1,?2,?3,?4,?5)",params![request.payload().digest.as_slice(),prepared.hash.as_slice(),request.raw(),verified.0,verified.1.as_slice()]).map_err(|_|Failure::WriteFailed)?;
    db.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)?;drop(db);
    let db=adapter.connect()?;db.execute_batch("BEGIN").map_err(|_|Failure::RereadFailed)?;
    check_prefix(&db,prepared,Some(verified),false)?;db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)
}
fn commit_source_head(adapter:&SqliteAdapterV1,prepared:&PreparedFirstSourceWorldV2,verified:&(Vec<u8>,[u8;32]))->Result<(),Failure>{
    let db=adapter.connect()?;db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::AtomicCommitFailed)?;
    let result=(||->Result<(),Failure>{
        check_prefix(&db,prepared,Some(verified),false)?;check_source_head(&db,&prepared.start)?;
        #[cfg(test)]
        if SOURCE_WORLD_FAULT.with(|v|v.get())==1{return Err(Failure::AtomicCommitFailed);}
        let p=prepared.start.request().payload();let b=&p.binding;let old=prepared.start.expected();
        db.execute("INSERT INTO vc04_project_revisions VALUES (?1,?2,?3,1,?4)",params![b.project,b.new_revision,b.revision_integrity.as_slice(),p.digest.as_slice()]).map_err(|_|Failure::ConflictingRetry)?;
        let changed=db.execute("UPDATE vc04_project_heads SET history_revision_identity=?1,revision_integrity_sha256=?2,payload_kind=1,payload_key=?3 WHERE project_identity=?4 AND history_revision_identity=?5 AND revision_integrity_sha256=?6 AND payload_kind=?7 AND payload_key=?8",
            params![b.new_revision,b.revision_integrity.as_slice(),p.digest.as_slice(),b.project,old.revision,old.integrity.as_slice(),if prepared.start.is_first(){0}else{1},old.digest.as_slice()]).map_err(|_|Failure::AtomicCommitFailed)?;
        if changed!=1{return Err(Failure::StaleRevision);}Ok(())
    })();
    if let Err(cause)=result{return match db.execute_batch("ROLLBACK"){Ok(())=>Err(cause),Err(_)=>Err(Failure::RecoveryRequired)};}
    db.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)
}
fn read_chain(db:&rusqlite::Connection,payload:&StoredPayloadV1)->Result<super::RawChainV1,Failure>{
    fn row(db:&rusqlite::Connection,table:&str,p:&StoredPayloadV1,revision:bool)->Result<Option<super::StoredHeadV1>,Failure>{
        // Only fixed internal table names enter this function.
        let sql=format!("SELECT project_identity,history_revision_identity,revision_integrity_sha256,payload_key,payload_kind FROM {table} WHERE project_identity=?1{}",if revision{" AND history_revision_identity=?2"}else{""});
        let decode=|r:&rusqlite::Row<'_>|->rusqlite::Result<super::StoredHeadV1>{
            if r.get::<_,i64>(4)?!=1{return Err(rusqlite::Error::InvalidQuery);}
            for i in [0,1]{let b=r.get_ref(i)?.as_blob()?;if b.is_empty()||b.len()>4096{return Err(rusqlite::Error::InvalidQuery);}}
            for i in [2,3]{if r.get_ref(i)?.as_blob()?.len()!=32{return Err(rusqlite::Error::InvalidQuery);}}
            Ok(super::StoredHeadV1{project:r.get(0)?,revision:r.get(1)?,integrity:r.get(2)?,digest:r.get(3)?})
        };
        if revision{db.query_row(&sql,params![p.binding.project,p.binding.new_revision],decode)}
        else{db.query_row(&sql,params![p.binding.project],decode)}.optional().map_err(super::database_error)
    }
    Ok(super::RawChainV1{payload:read_world(db,&payload.digest)?,head:row(db,"vc04_project_heads",payload,false)?,revision:row(db,"vc04_project_revisions",payload,true)?})
}

/// Exact first-attempt Committed prefix only. Retry epochs and uncertain
/// outcomes require their separate recovery owner, never a shortened prefix.
pub(super) fn check_cold_first_world(db:&rusqlite::Connection,source:&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>,digest:&[u8;32],request_hash:&[u8;32],token_hash:&[u8;32],witness_hash:&[u8;32])
    ->Result<StoredPayloadV1,Failure>{
    let payload=read_world(db,digest)?.ok_or(Failure::RecoveryRequired)?;
    let worlds:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_prepared_payloads WHERE payload_kind=1 LIMIT 2)",[],|r|r.get(0)).map_err(super::database_error)?;
    if worlds!=1||payload.binding.expected_parent!=source.revision()||payload.binding.project!=source.project()
        ||payload.binding.execution_profile!=source.profile()||payload.envelope!=context.revision_envelope()
        ||payload.binding.pesi!=*context.pesi().as_raw()
        ||crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(context.owned()).map_err(|_|Failure::CustodyConflict)?.as_bytes()!=source.payload(){
        return Err(Failure::CustodyConflict);
    }
    owner::checked_native_committed_payload(&payload,&read_chain(db,&payload)?)?;
    check_world_custody(db,&payload,source.integrity(),context,Some(request_hash),Some(token_hash),Some(witness_hash))?;
    Ok(payload)
}

/// Data validation only. Historical rows gain no current-head/Committed
/// authority here; only the protected current source owner may anchor them.
fn check_world_custody(db:&rusqlite::Connection,payload:&StoredPayloadV1,predecessor:&[u8;32],
    context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>,
    request_hash:Option<&[u8;32]>,token_hash:Option<&[u8;32]>,witness_hash:Option<&[u8;32]>)->Result<(),Failure>{
    use sha2::{Digest,Sha256};
    use owner::recovery_custody::RecoveryRecordCandidateV1;
    let digest=&payload.digest;
    if !super::custody_count_matches(db,digest,3)?{return Err(Failure::RecoveryRequired);}
    let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(digest);
    if token_hash.is_some_and(|expected|<[u8;32]>::from(Sha256::digest(token))!=*expected){return Err(Failure::CustodyConflict);}
    let mut expected_request=Vec::new();let mut previous=[0;32];let mut initial=Vec::new();
    for ordinal in 0u8..3 {
        let record=db.query_row("SELECT custody_version,previous_row_sha256,admitted_request_bytes,recovery_record_bytes,row_sha256 FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=?2",
            params![digest.as_slice(),ordinal],|r|{
                let req=r.get_ref(2)?.as_blob()?;let record=r.get_ref(3)?.as_blob()?;
                if req.is_empty()||req.len()>owner::recovery_custody::REQUEST_MAX||record.is_empty()||record.len()>owner::recovery_custody::RECORD_MAX
                    ||r.get::<_,i64>(0)?!=1||r.get_ref(1)?.as_blob()?!=previous{
                    return Err(rusqlite::Error::InvalidQuery);
                }
                if ordinal==0 {expected_request.extend_from_slice(req);}else if req!=expected_request{return Err(rusqlite::Error::InvalidQuery);}
                let hash=InitialCustodyRowV1::row_hash(digest,ordinal,&previous,req,record).map_err(|_|rusqlite::Error::InvalidQuery)?;
                if r.get_ref(4)?.as_blob()?!=hash{return Err(rusqlite::Error::InvalidQuery);}
                previous=hash;Ok(record.to_vec())
            }).map_err(super::database_error)?;
        let decoded=RecoveryRecordCandidateV1::decode(&record)?;decoded.check_binding(&payload.binding,Some(predecessor))?;
        if decoded.encode_canonical()?!=record{return Err(Failure::CorruptRecoveryRecord);}
        match ordinal {
            0=>{if record!=encode_prepared_candidate(&payload.binding,Some(predecessor))?{return Err(Failure::CustodyConflict);}initial=record;},
            1=>if record!=encode_verified_candidate(&initial)?{return Err(Failure::CustodyConflict);},
            // The existing common owner CONSUMES its fresh confirmation when
            // entering Committed. Historical field14 is therefore zero, not
            // a reusable confirmation token. Cold confirmation occurred above.
            2=>if decoded.state()!=State::Committed||decoded.cause()!=Failure::Interrupted||decoded.confirmed()
                ||witness_hash.is_some_and(|expected|<[u8;32]>::from(Sha256::digest(&record))!=*expected){return Err(Failure::RecoveryRequired);},
            _=>unreachable!(),
        }
    }
    if request_hash.is_some_and(|expected|<[u8;32]>::from(Sha256::digest(&expected_request))!=*expected){return Err(Failure::CustodyConflict);}
    let catalog=context.owned().registration().map(|registration|registration.saved_nominal_catalog(context.owned()))
        .transpose().map_err(|_|Failure::CustodyConflict)?;
    let checked=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2_with_catalog(&expected_request,context.pesi().as_raw(),catalog.as_ref())?;
    if checked.request_kind!=1||checked.binding!=payload.binding||checked.snapshot_bytes!=payload.snapshot
        ||checked.revision_bytes!=payload.envelope||checked.encode_canonical_with_catalog(catalog.as_ref())?!=expected_request{return Err(Failure::CustodyConflict);}
    Ok(())
}

/// Anchors the complete immutable prefix to the caller's actual protected
/// current Witness. No historical head is synthesized and no MAX-row lookup
/// or receipt alone selects the latest revision.
pub(super) fn check_cold_source_world_history(db:&rusqlite::Connection,
    source:&crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision,
    context:&crate::unit_wire::project_runtime_attachment::SourceEnvelopeBoundRuntimeAttachment<'_>,
    digest:&[u8;32],request_hash:&[u8;32],token_hash:&[u8;32],witness_hash:&[u8;32],epoch:u64)
    ->Result<(StoredPayloadV1,Vec<[u8;32]>),Failure>{
    use crate::unit_wire::resumable_complete_state as complete;
    let current=read_world(db,digest)?.ok_or(Failure::RecoveryRequired)?;
    owner::checked_native_committed_payload(&current,&read_chain(db,&current)?)?;
    if current.envelope!=context.revision_envelope()||current.binding.project!=source.project()
        ||current.binding.pesi!=*context.pesi().as_raw()||current.binding.execution_profile!=source.profile()
        ||crate::unit_wire::project_runtime_attachment::saved_runtime_project_payload_json(context.owned()).map_err(|_|Failure::CustodyConflict)?.as_bytes()!=source.payload(){return Err(Failure::CustodyConflict);}
    let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&current.snapshot,context.pesi().as_raw())?;
    let descriptor=complete::rebuild_source_descriptor_candidate(layout.supplies[9],context.owned())?;
    if epoch!=u64::try_from(descriptor.prior_durable_receipts.len()).map_err(|_|Failure::LimitExceeded)?.checked_add(1).ok_or(Failure::LimitExceeded)?{return Err(Failure::CustodyConflict);}
    let count:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_prepared_payloads WHERE payload_kind=1 LIMIT 65537)",[],|r|r.get(0)).map_err(super::database_error)?;
    if epoch>65536||u64::try_from(count).ok()!=Some(epoch){return Err(Failure::CustodyConflict);}
    let revisions:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_project_revisions WHERE payload_kind=1 LIMIT 65537)",[],|r|r.get(0)).map_err(super::database_error)?;
    let custody:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_recovery_custody_v1 LIMIT 196609)",[],|r|r.get(0)).map_err(super::database_error)?;
    if revisions!=count||custody!=count.checked_mul(3).ok_or(Failure::LimitExceeded)?{return Err(Failure::CustodyConflict);}
    let mut digests=Vec::new();let mut parent=source.revision().to_vec();let mut predecessor=*source.integrity();
    for index in 0..=descriptor.prior_durable_receipts.len(){
        let old=if index<descriptor.prior_durable_receipts.len(){
            let receipt=crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(descriptor.prior_durable_receipts[index])?;
            let head=receipt.native_head()?;
            let payload=read_world(db,&head.digest)?.ok_or(Failure::RecoveryRequired)?;
            receipt.check_binding(&payload.binding,b"vc04.sqlite-durability.r1")?;
            let chain=read_chain(db,&payload)?;
            let revision=chain.revision.ok_or(Failure::RecoveryRequired)?;
            if revision.project!=head.project||revision.revision!=head.revision||revision.integrity!=head.integrity||revision.digest!=head.digest{return Err(Failure::CustodyConflict);}
            payload
        }else{current.clone()};
        if old.binding.expected_parent!=parent||old.binding.project!=current.binding.project||old.binding.pesi!=current.binding.pesi
            ||old.binding.execution_profile!=current.binding.execution_profile||digests.contains(&old.digest){return Err(Failure::CustodyConflict);}
        let old_layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&old.snapshot,context.pesi().as_raw())?;
        let old_descriptor=complete::rebuild_source_descriptor_candidate(old_layout.supplies[9],context.owned())?;
        if old_descriptor.source_descriptor_prefix!=descriptor.source_descriptor_prefix||old_descriptor.execution_instance!=descriptor.execution_instance
            ||old_descriptor.prior_durable_receipts!=descriptor.prior_durable_receipts[..index]{return Err(Failure::CustodyConflict);}
        let latest=index==descriptor.prior_durable_receipts.len();
        check_world_custody(db,&old,&predecessor,context,latest.then_some(request_hash),latest.then_some(token_hash),latest.then_some(witness_hash))?;
        parent=old.binding.new_revision.clone();predecessor=old.digest;digests.push(old.digest);
    }
    Ok((current,digests))
}

pub(super) fn read_source_history_payload(db:&rusqlite::Connection,digest:&[u8;32])->Result<StoredPayloadV1,Failure>{
    read_world(db,digest)?.ok_or(Failure::RecoveryRequired)
}
fn check_source_head(db:&rusqlite::Connection,start:&SourceWorldStart)->Result<(),Failure>{
    let expected=start.expected();
    let matches:bool=db.query_row("SELECT history_revision_identity=?2 AND revision_integrity_sha256=?3 AND payload_kind=?4 AND payload_key=?5 FROM vc04_project_heads WHERE project_identity=?1",
        params![expected.project,expected.revision,expected.integrity.as_slice(),if start.is_first(){0}else{1},expected.digest.as_slice()],|r|r.get(0)).optional()
        .map_err(|_|Failure::RereadFailed)?.unwrap_or(false);
    if !matches{return Err(Failure::StaleRevision);}Ok(())
}
impl SqliteAdapterV1 {
    pub(crate) fn write_first_source_world_prepared(&self,start:SourceWorldAttemptStartV2,
        context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<PreparedFirstSourceWorldV2,Failure>{
        self.write_source_world_prepared(SourceWorldStart::First(start),context)
    }
    pub(crate) fn write_next_source_world_prepared(&self,start:NextSourceWorldAttemptStartV1,
        context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<PreparedFirstSourceWorldV2,Failure>{
        self.write_source_world_prepared(SourceWorldStart::Next(start),context)
    }
    fn write_source_world_prepared(&self,start:SourceWorldStart,
        context:&SourceBoundSavedRuntimeAttachment<'_>)->Result<PreparedFirstSourceWorldV2,Failure>{
        if self.schema_profile!=NativeSchemaProfile::SourceRevision{return Err(Failure::UnsupportedDurability);}
        start.check_current(self,context)?;
        let payload=start.request().payload();let b=&payload.binding;
        if payload.digest!=b.transaction_digest()?||payload.snapshot.len()>2_000_000
            ||payload.snapshot.is_empty()||payload.envelope.is_empty()||payload.envelope.len()>1_000_000{
            return Err(Failure::LimitExceeded);
        }
        let record=encode_prepared_candidate(b,Some(&start.expected().digest))?;
        let hash=InitialCustodyRowV1::row_hash(&payload.digest,0,&[0;32],start.request().raw(),&record)?;
        let db=self.connect()?;db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::WriteFailed)?;
        let written=(||->Result<(),Failure>{
            check_source_head(&db,&start)?;
            let worlds:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_prepared_payloads WHERE payload_kind=1 LIMIT 2)",[],|r|r.get(0)).map_err(|_|Failure::RereadFailed)?;
            let custody:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_recovery_custody_v1 LIMIT 2)",[],|r|r.get(0)).map_err(|_|Failure::RereadFailed)?;
            if start.is_first()&&(worlds!=0||custody!=0){return Err(Failure::ConflictingRetry);}
            db.execute("INSERT INTO vc04_prepared_payloads VALUES (1,?1,?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL,?10,?11)",
                params![payload.digest.as_slice(),b.project,b.expected_parent,b.new_revision,b.execution_profile,b.pesi.as_slice(),
                    payload.snapshot,i64::try_from(b.snapshot_length).map_err(|_|Failure::LimitExceeded)?,b.snapshot_sha256.as_slice(),b.revision_integrity.as_slice(),payload.envelope])
                .map_err(|_|Failure::WriteFailed)?;
            db.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,0,1,?2,?3,?4,?5)",
                params![payload.digest.as_slice(),[0u8;32].as_slice(),start.request().raw(),record,hash.as_slice()])
                .map_err(|_|Failure::WriteFailed)?;
            Ok(())
        })();
        if let Err(cause)=written{return match db.execute_batch("ROLLBACK"){Ok(())=>Err(cause),Err(_)=>Err(Failure::RecoveryRequired)};}
        db.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)?;drop(db);
        self.confirm_container_durable()?;
        let prepared=PreparedFirstSourceWorldV2{start,record,hash};prepared.check_current(self,context)?;Ok(prepared)
    }
}

/// Explicit world-only projection of the new schema. Inspect borrowed BLOBs
/// before allocations; neither source payload nor arbitrary JSON is a frame.
pub(super) fn read_world(db:&rusqlite::Connection,digest:&[u8;32])->Result<Option<StoredPayloadV1>,Failure>{
    db.query_row("SELECT transaction_digest,project_identity,expected_parent_history_revision_identity,new_history_revision_identity,
        execution_profile_identity,project_execution_snapshot_identity,canonical_snapshot_v2_bytes,snapshot_v2_byte_length_u64,
        snapshot_v2_sha256,revision_integrity_sha256,revision_envelope_bytes,payload_key,project_payload_bytes
        FROM vc04_prepared_payloads WHERE payload_kind=1 AND transaction_digest=?1",[digest.as_slice()],|r|{
        let blob=|i,max|->rusqlite::Result<&[u8]>{let v=r.get_ref(i)?.as_blob()?;
            if v.is_empty()||v.len()>max{return Err(rusqlite::Error::InvalidQuery);}Ok(v)};
        for i in [0,5,8,9,11]{if blob(i,32)?.len()!=32{return Err(rusqlite::Error::InvalidQuery);}}
        for i in [1,2,3,4]{blob(i,4096)?;}
        let length:i64=r.get(7)?;
        if length<=0||length>2_000_000||blob(6,2_000_000)?.len() as i64!=length
            ||blob(11,32)?!=digest||blob(0,32)?!=digest||!matches!(r.get_ref(12)?,rusqlite::types::ValueRef::Null){
            return Err(rusqlite::Error::InvalidQuery);
        }
        blob(10,1_000_000)?;
        Ok(StoredPayloadV1{digest:r.get(0)?,binding:TransactionBindingV1{project:r.get(1)?,expected_parent:r.get(2)?,
            new_revision:r.get(3)?,execution_profile:r.get(4)?,pesi:r.get(5)?,snapshot_length:length as u64,
            snapshot_sha256:r.get(8)?,revision_integrity:r.get(9)?},snapshot:r.get(6)?,envelope:r.get(10)?})
    }).optional().map_err(super::database_error)
}
