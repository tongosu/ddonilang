//! R2 append-only Intent bytes. A SQL row is not recovery or Ready authority.
use super::*;
use crate::replay_evidence::NativeFinishRecoveryIntentV1;
use crate::replay_evidence::finish_recovery::{RecordHeaderV1,RecordKindV1,CHUNK_BYTES,HEADER_LIMIT};
use sha2::{Digest,Sha256};
mod cold;
pub(in crate::snapshot_v2_durable_transaction::native) use cold::FinishRecoveryHeadersV1;

struct Key {first:[u8;32],ordinal:i64,epoch:i64}
fn check_scope(adapter:&SqliteAdapterV1,scope:&ReplayArchiveRootScopeV1,
    intent:&NativeFinishRecoveryIntentV1)->Result<Key,Failure>{
    let header=RecordHeaderV1::parse(intent.header_bytes()).map_err(|_|Failure::InvalidInput)?;
    if header.kind()!=RecordKindV1::Intent{return Err(Failure::InvalidInput);}
    let (project,nonce,fingerprint,generation)=scope.identity_fields(adapter)?;
    let generation=generation.to_be_bytes();
    for (tag,expected) in [(1,project.as_slice()),(2,nonce.as_slice()),(3,fingerprint.as_slice()),(4,generation.as_slice())]{
        if header.field(tag)!=Some(expected){return Err(Failure::CustodyConflict);}
    }
    let number=|tag|->Result<i64,Failure>{
        let raw:[u8;8]=header.field(tag).ok_or(Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        i64::try_from(u64::from_be_bytes(raw)).map_err(|_|Failure::LimitExceeded)
    };
    let first=header.field(5).ok_or(Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
    let key=Key{first,ordinal:number(7)?,epoch:number(6)?};
    if key.ordinal<=0{return Err(Failure::InvalidInput);}
    let digest=header.check_payload(intent.payload_bytes().chunks(CHUNK_BYTES)).map_err(|_|Failure::InvalidInput)?;
    if &digest!=intent.record_digest(){return Err(Failure::ArtifactMismatch);}
    Ok(key)
}

fn verify(db:&Connection,key:&Key,intent:&NativeFinishRecoveryIntentV1)->Result<(),Failure>{
    // Borrow SQLite blobs and bound each before copying. Never collect the
    // declared whole payload or trust a row's length as allocation authority.
    let metadata=db.query_row(
        "SELECT recording_epoch,kind,header_bytes,payload_length,chunk_count,previous_evidence_sha256,record_sha256 FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=0",
        params![key.first.as_slice(),key.ordinal],|row|{
            let header=row.get_ref(2)?.as_blob()?;
            let previous=row.get_ref(5)?.as_blob()?;let digest=row.get_ref(6)?.as_blob()?;
            if header.len()>HEADER_LIMIT || previous.len()!=32 || digest.len()!=32{return Err(rusqlite::Error::InvalidQuery);}
            Ok((row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,header.to_vec(),row.get::<_,i64>(3)?,
                row.get::<_,i64>(4)?,previous.to_vec(),digest.to_vec()))
        }).map_err(database_error)?;
    let expected_chunks=intent.payload_bytes().len().div_ceil(CHUNK_BYTES);
    if metadata.0!=key.epoch || metadata.1!=0 || metadata.2.as_slice()!=intent.header_bytes()
        || usize::try_from(metadata.3).ok()!=Some(intent.payload_bytes().len())
        || usize::try_from(metadata.4).ok()!=Some(expected_chunks)
        || metadata.5.as_slice()!=[0u8;32].as_slice()
        || metadata.6.as_slice()!=intent.record_digest().as_slice(){return Err(Failure::ConflictingRetry);}
    let mut statement=db.prepare("SELECT chunk_ordinal,chunk_bytes FROM wo25_finish_recovery_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=0 ORDER BY chunk_ordinal").map_err(database_error)?;
    let mut rows=statement.query(params![key.first.as_slice(),key.ordinal]).map_err(database_error)?;
    let mut digest=Sha256::new();digest.update(intent.header_bytes());
    for (index,expected) in intent.payload_bytes().chunks(CHUNK_BYTES).enumerate(){
        let row=rows.next().map_err(database_error)?.ok_or(Failure::RereadMismatch)?;
        if usize::try_from(row.get::<_,i64>(0).map_err(database_error)?).ok()!=Some(index){return Err(Failure::RereadMismatch);}
        let bytes=row.get_ref(1).map_err(database_error)?.as_blob().map_err(|_|Failure::RereadMismatch)?;
        if bytes.is_empty()||bytes.len()>CHUNK_BYTES||bytes!=expected{return Err(Failure::RereadMismatch);}
        digest.update(bytes);
    }
    if rows.next().map_err(database_error)?.is_some() || digest.finalize().as_slice()!=intent.record_digest(){return Err(Failure::RereadMismatch);}
    Ok(())
}

impl SqliteAdapterV1 {
    pub(crate) fn complete_finish_recovery_ready(&mut self,
        plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,intent:&NativeFinishRecoveryIntentV1,
        pending:crate::replay_evidence::PendingReplayLifecycleV1,
        context:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)
        ->Result<ProtectedReplayReadyV1,super::replay_archive_schema::CapturedReplayMetadataFailureV1>{
        use super::replay_archive_schema::CapturedReplayMetadataFailureV1 as Captured;
        let actual=self.revalidate_replay_scope_current_world(&plan.scope,context)
            .map_err(|cause|Captured::finish_native(14,Some(cause),
                format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 원 확정 거래 재검증 ({cause:?})"),intent,false))?;
        let ready=pending.verify_metadata_anchor(plan,intent.row(),self,&actual,context)
            .map_err(|message|Captured::finish_native(14,None,
                format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: {message}"),intent,false))?;
        plan.scope.write_finish_ready(self,intent,&ready)
    }

    /// Explicit profile selection is required before this call. Ordinary
    /// readers never create tables. A complete existing Intent is only read.
    pub(crate) fn append_finish_recovery_intent(&mut self,scope:&ReplayArchiveRootScopeV1,
        intent:&NativeFinishRecoveryIntentV1)->Result<(),Failure>{
        let key=check_scope(self,scope,intent)?;
        let mut db=self.connect()?;
        let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(database_error)?;
        let count:i64=transaction.query_row("SELECT count(*) FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=0",
            params![key.first.as_slice(),key.ordinal],|row|row.get(0)).map_err(database_error)?;
        if count==0{
            transaction.execute("INSERT INTO wo25_finish_recovery_records_v1(execution_first_tx,control_ordinal,recording_epoch,evidence_ordinal,kind,header_bytes,payload_length,chunk_count,previous_evidence_sha256,record_sha256) VALUES(?1,?2,?3,0,0,?4,?5,?6,?7,?8)",
                params![key.first.as_slice(),key.ordinal,key.epoch,intent.header_bytes(),intent.payload_bytes().len() as i64,
                    intent.payload_bytes().len().div_ceil(CHUNK_BYTES) as i64,[0u8;32].as_slice(),intent.record_digest().as_slice()]).map_err(database_error)?;
            let mut insert=transaction.prepare("INSERT INTO wo25_finish_recovery_chunks_v1(execution_first_tx,control_ordinal,evidence_ordinal,chunk_ordinal,chunk_bytes) VALUES(?1,?2,0,?3,?4)").map_err(database_error)?;
            for (index,chunk) in intent.payload_bytes().chunks(CHUNK_BYTES).enumerate(){
                insert.execute(params![key.first.as_slice(),key.ordinal,index as i64,chunk]).map_err(database_error)?;
            }
        }else if count!=1{return Err(Failure::ConflictingRetry);}
        verify(&transaction,&key,intent)?;
        transaction.commit().map_err(database_error)?;drop(db);
        self.confirm_container_durable()?;
        // Fresh handle/read transaction, then live root/current/lease again.
        self.verify_finish_recovery_intent(scope,intent)
    }
    pub(crate) fn verify_finish_recovery_intent(&self,scope:&ReplayArchiveRootScopeV1,
        intent:&NativeFinishRecoveryIntentV1)->Result<(),Failure>{
        let key=check_scope(self,scope,intent)?;let mut db=self.connect()?;
        let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred).map_err(database_error)?;
        verify(&transaction,&key,intent)?;
        transaction.commit().map_err(database_error)?;drop(db);scope.verify_current(self)
    }

    /// Only the actual captured metadata failure can supply record1. An
    /// arbitrary message/code/hash is not an accepted producer argument.
    pub(crate) fn append_finish_recovery_failure(&mut self,scope:&ReplayArchiveRootScopeV1,
        intent:&NativeFinishRecoveryIntentV1,
        captured:&super::replay_archive_schema::CapturedReplayMetadataFailureV1,
        witness:&super::root_grant::FinishFailureCaptureWitnessV1)->Result<[u8;32],Failure>{
        scope.verify_finish_failure_capture(self,intent,captured,witness)?;
        let key=check_scope(self,scope,intent)?;
        let (header,payload)=captured.record_for(intent).map_err(|_|Failure::InvalidInput)?;
        let digest=RecordHeaderV1::parse(&header).map_err(|_|Failure::InvalidInput)?
            .check_payload([payload]).map_err(|_|Failure::InvalidInput)?;
        let mut db=self.connect()?;
        let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(database_error)?;
        verify(&transaction,&key,intent)?;
        let count:i64=transaction.query_row("SELECT count(*) FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1",
            params![key.first.as_slice(),key.ordinal],|row|row.get(0)).map_err(database_error)?;
        if count==0{
            transaction.execute("INSERT INTO wo25_finish_recovery_records_v1(execution_first_tx,control_ordinal,recording_epoch,evidence_ordinal,kind,header_bytes,payload_length,chunk_count,previous_evidence_sha256,record_sha256) VALUES(?1,?2,?3,1,1,?4,?5,1,?6,?7)",
                params![key.first.as_slice(),key.ordinal,key.epoch,&header,payload.len() as i64,
                    intent.record_digest().as_slice(),digest.as_slice()]).map_err(database_error)?;
            transaction.execute("INSERT INTO wo25_finish_recovery_chunks_v1(execution_first_tx,control_ordinal,evidence_ordinal,chunk_ordinal,chunk_bytes) VALUES(?1,?2,1,0,?3)",
                params![key.first.as_slice(),key.ordinal,payload]).map_err(database_error)?;
        }else if count!=1{return Err(Failure::ConflictingRetry);}
        verify_failure(&transaction,&key,intent,&header,payload,&digest)?;
        transaction.commit().map_err(database_error)?;drop(db);
        self.confirm_container_durable()?;
        self.verify_finish_recovery_failure(scope,intent,captured)?;
        scope.verify_finish_failure_capture(self,intent,captured,witness)?;
        Ok(digest)
    }

    pub(crate) fn verify_finish_recovery_failure(&self,scope:&ReplayArchiveRootScopeV1,
        intent:&NativeFinishRecoveryIntentV1,
        captured:&super::replay_archive_schema::CapturedReplayMetadataFailureV1)->Result<[u8;32],Failure>{
        let key=check_scope(self,scope,intent)?;
        let (header,payload)=captured.record_for(intent).map_err(|_|Failure::InvalidInput)?;
        let digest=RecordHeaderV1::parse(&header).map_err(|_|Failure::InvalidInput)?
            .check_payload([payload]).map_err(|_|Failure::InvalidInput)?;
        let mut db=self.connect()?;
        let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred).map_err(database_error)?;
        verify(&transaction,&key,intent)?;
        verify_failure(&transaction,&key,intent,&header,payload,&digest)?;
        transaction.commit().map_err(database_error)?;drop(db);
        scope.verify_current(self)?;Ok(digest)
    }
}

fn verify_failure(db:&Connection,key:&Key,intent:&NativeFinishRecoveryIntentV1,
    header:&[u8],payload:&[u8],digest:&[u8;32])->Result<(),Failure>{
    // Bounded expected bytes came from the actual producer. SQL compares all
    // stored bytes without allocating an attacker-declared blob in Rust.
    let count:i64=db.query_row("SELECT count(*) FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND recording_epoch=?3 AND evidence_ordinal=1 AND kind=1 AND header_bytes=?4 AND payload_length=?5 AND chunk_count=1 AND previous_evidence_sha256=?6 AND record_sha256=?7",
        params![key.first.as_slice(),key.ordinal,key.epoch,header,payload.len() as i64,
            intent.record_digest().as_slice(),digest.as_slice()],|row|row.get(0)).map_err(database_error)?;
    if count!=1{return Err(Failure::ConflictingRetry);}
    let count:i64=db.query_row("SELECT count(*) FROM wo25_finish_recovery_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1",
        params![key.first.as_slice(),key.ordinal],|row|row.get(0)).map_err(database_error)?;
    if count!=1{return Err(Failure::RereadMismatch);}
    let count:i64=db.query_row("SELECT count(*) FROM wo25_finish_recovery_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=1 AND chunk_ordinal=0 AND chunk_bytes=?3",
        params![key.first.as_slice(),key.ordinal,payload],|row|row.get(0)).map_err(database_error)?;
    if count!=1{return Err(Failure::RereadMismatch);}
    Ok(())
}
