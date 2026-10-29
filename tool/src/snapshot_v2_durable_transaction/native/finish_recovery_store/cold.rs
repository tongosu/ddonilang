//! Exact cold SQL data only. No Native Intent issuer, Ready or repair permit.
use super::*;
use crate::replay_evidence::{ReplayLifecycleCandidateV1,finish_recovery::PAYLOAD_LIMIT};

pub(in crate::snapshot_v2_durable_transaction::native) struct StoredFinishHeaderV1 {
    raw:Vec<u8>,digest:[u8;32],previous:[u8;32],epoch:i64,kind:i64,
}
impl StoredFinishHeaderV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) fn raw(&self)->&[u8]{&self.raw}
    pub(in crate::snapshot_v2_durable_transaction::native) fn digest(&self)->&[u8;32]{&self.digest}
}
pub(in crate::snapshot_v2_durable_transaction::native) struct FinishRecoveryHeadersV1 {
    key:Key,intent:StoredFinishHeaderV1,failure:Option<StoredFinishHeaderV1>,
}
impl FinishRecoveryHeadersV1 {
    pub(in crate::snapshot_v2_durable_transaction::native) fn intent(&self)->&StoredFinishHeaderV1{&self.intent}
    pub(in crate::snapshot_v2_durable_transaction::native) fn failure(&self)->Option<&StoredFinishHeaderV1>{self.failure.as_ref()}
    pub(in crate::snapshot_v2_durable_transaction::native) fn same_bytes(&self,other:&Self)->bool{
        self.key.first==other.key.first && self.key.ordinal==other.key.ordinal && self.key.epoch==other.key.epoch
            && same_header(&self.intent,&other.intent) && match (&self.failure,&other.failure){
                (None,None)=>true,(Some(a),Some(b))=>same_header(a,b),_=>false,
            }
    }
}
fn corrupt()->rusqlite::Error{rusqlite::Error::InvalidQuery}
fn bounded_copy(raw:&[u8],cap:usize)->rusqlite::Result<Vec<u8>>{
    if raw.is_empty()||raw.len()>cap{return Err(corrupt());}
    let mut copy=Vec::new();copy.try_reserve_exact(raw.len()).map_err(|_|corrupt())?;
    copy.extend_from_slice(raw);Ok(copy)
}
fn read_header(db:&Connection,key:&Key,evidence:i64)->Result<StoredFinishHeaderV1,Failure>{
    let (epoch,kind,raw,length,chunks,previous,digest)=db.query_row(
        "SELECT recording_epoch,kind,header_bytes,payload_length,chunk_count,previous_evidence_sha256,record_sha256 FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=?3",
        params![key.first.as_slice(),key.ordinal,evidence],|row|{
            let previous=row.get_ref(5)?.as_blob()?.try_into().map_err(|_|corrupt())?;
            let digest=row.get_ref(6)?.as_blob()?.try_into().map_err(|_|corrupt())?;
            Ok((row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,bounded_copy(row.get_ref(2)?.as_blob()?,HEADER_LIMIT)?,
                row.get::<_,i64>(3)?,row.get::<_,i64>(4)?,previous,digest))
        }).map_err(database_error)?;
    let header=RecordHeaderV1::parse(&raw).map_err(|_|Failure::CorruptRecoveryRecord)?;
    let expected=if evidence==0{RecordKindV1::Intent}else{RecordKindV1::ActualFailure};
    if epoch!=key.epoch || kind!=evidence || header.kind()!=expected
        || u64::try_from(length).ok()!=Some(header.payload_length())
        || u32::try_from(chunks).ok()!=Some(header.chunk_count()){
        return Err(Failure::CorruptRecoveryRecord);
    }
    Ok(StoredFinishHeaderV1{raw,digest,previous,epoch,kind})
}
fn same_header(a:&StoredFinishHeaderV1,b:&StoredFinishHeaderV1)->bool{
    a.raw==b.raw && a.digest==b.digest && a.previous==b.previous && a.epoch==b.epoch && a.kind==b.kind
}

/// Hash first without allocating the claimed whole payload. The second pass
/// uses the same SQLite snapshot only after the first full digest matched.
fn payload_pass(db:&Connection,key:&Key,stored:&StoredFinishHeaderV1,collect:bool)->Result<Vec<u8>,Failure>{
    let header=RecordHeaderV1::parse(&stored.raw).map_err(|_|Failure::CorruptRecoveryRecord)?;
    let length=usize::try_from(header.payload_length()).map_err(|_|Failure::LimitExceeded)?;
    if length as u64>PAYLOAD_LIMIT{return Err(Failure::LimitExceeded);}
    let mut payload=Vec::new();
    if collect{payload.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;}
    let mut statement=db.prepare("SELECT chunk_ordinal,chunk_bytes FROM wo25_finish_recovery_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND evidence_ordinal=?3 ORDER BY chunk_ordinal").map_err(database_error)?;
    let mut rows=statement.query(params![key.first.as_slice(),key.ordinal,stored.kind]).map_err(database_error)?;
    let mut record=Sha256::new();record.update(&stored.raw);let mut body=Sha256::new();
    for index in 0..header.chunk_count(){
        let row=rows.next().map_err(database_error)?.ok_or(Failure::CorruptRecoveryRecord)?;
        if row.get::<_,i64>(0).map_err(database_error)?!=i64::from(index){return Err(Failure::CorruptRecoveryRecord);}
        let bytes=row.get_ref(1).map_err(database_error)?.as_blob().map_err(|_|Failure::CorruptRecoveryRecord)?;
        let expected=if index+1<header.chunk_count(){CHUNK_BYTES}
            else{length.checked_sub(index as usize*CHUNK_BYTES).ok_or(Failure::CorruptRecoveryRecord)?};
        if bytes.is_empty()||bytes.len()>CHUNK_BYTES||bytes.len()!=expected{return Err(Failure::CorruptRecoveryRecord);}
        if header.kind()==RecordKindV1::ActualFailure{std::str::from_utf8(bytes).map_err(|_|Failure::CorruptRecoveryRecord)?;}
        body.update(bytes);record.update(bytes);if collect{payload.extend_from_slice(bytes);}
    }
    let tag=if header.kind()==RecordKindV1::Intent{18}else{5};
    if rows.next().map_err(database_error)?.is_some() || Some(body.finalize().as_slice())!=header.field(tag)
        || record.finalize().as_slice()!=stored.digest.as_slice(){return Err(Failure::CorruptRecoveryRecord);}
    Ok(payload)
}

impl SqliteAdapterV1 {
    /// Selection comes from the current protected LC of an actual cold world
    /// proof. This function returns data, never a repair/execution admission.
    pub(in crate::snapshot_v2_durable_transaction::native) fn read_finish_recovery_headers(
        &self,proof:&VerifiedExecutionFirstCheckpointV1,lifecycle:&[u8])->Result<FinishRecoveryHeadersV1,Failure>{
        proof.scope.verify_current(self)?;
        let life=ReplayLifecycleCandidateV1::parse(lifecycle).map_err(|_|Failure::CorruptRecoveryRecord)?;
        let number=|tag|->Result<i64,Failure>{
            let raw=life.payload(tag).ok_or(Failure::CorruptRecoveryRecord)?;
            i64::try_from(u64::from_be_bytes(raw.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?)).map_err(|_|Failure::LimitExceeded)
        };
        let key=Key{first:proof.first_tx,epoch:number(6)?,ordinal:number(7)?};
        if key.ordinal<=0 || life.payload(5)!=Some(key.first.as_slice()) || life.payload(9)!=Some(&[1][..])
            || life.payload(10)!=Some(proof.checkpoint_tx.as_slice()){
            return Err(Failure::CustodyConflict);
        }
        let mut db=self.connect()?;let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred).map_err(database_error)?;
        let count:i64=transaction.query_row("SELECT count(*) FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2",
            params![key.first.as_slice(),key.ordinal],|row|row.get(0)).map_err(database_error)?;
        if !(1..=2).contains(&count){return Err(Failure::RecoveryRequired);}
        let intent=read_header(&transaction,&key,0)?;
        let header=RecordHeaderV1::parse(&intent.raw).map_err(|_|Failure::CorruptRecoveryRecord)?;
        let (project,nonce,fingerprint,generation)=proof.scope.identity_fields(self)?;
        let generation=generation.to_be_bytes();let epoch=(key.epoch as u64).to_be_bytes();let ordinal=(key.ordinal as u64).to_be_bytes();
        for (tag,expected) in [(1,project.as_slice()),(2,nonce.as_slice()),(3,fingerprint.as_slice()),(4,generation.as_slice()),
            (5,key.first.as_slice()),(6,epoch.as_slice()),(7,ordinal.as_slice()),(8,proof.checkpoint_tx.as_slice())]{
            if header.field(tag)!=Some(expected){return Err(Failure::CustodyConflict);}
        }
        let life_tag=match life.payload(8){Some([0])=>10,Some([1])=>11,_=>return Err(Failure::CorruptRecoveryRecord)};
        if header.field(life_tag)!=Some(lifecycle)||intent.previous!=[0;32]{return Err(Failure::CustodyConflict);}
        let failure=if count==2{
            let failure=read_header(&transaction,&key,1)?;
            let parsed=RecordHeaderV1::parse(&failure.raw).map_err(|_|Failure::CorruptRecoveryRecord)?;
            if failure.previous!=intent.digest || parsed.field(1)!=Some(intent.digest.as_slice()){
                return Err(Failure::CustodyConflict);
            }Some(failure)
        }else{None};
        transaction.commit().map_err(database_error)?;drop(db);proof.scope.verify_current(self)?;
        Ok(FinishRecoveryHeadersV1{key,intent,failure})
    }

    /// Caller first checks the protected witness against these header hashes.
    /// Two passes share one snapshot, and both reject incomplete/extra chunks.
    pub(in crate::snapshot_v2_durable_transaction::native) fn read_finish_recovery_payloads(
        &self,proof:&VerifiedExecutionFirstCheckpointV1,headers:&FinishRecoveryHeadersV1)
        ->Result<(Vec<u8>,Option<Vec<u8>>),Failure>{
        proof.scope.verify_current(self)?;
        let mut db=self.connect()?;let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Deferred).map_err(database_error)?;
        if !same_header(&read_header(&transaction,&headers.key,0)?,&headers.intent){return Err(Failure::CustodyConflict);}
        let count:i64=transaction.query_row("SELECT count(*) FROM wo25_finish_recovery_records_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2",
            params![headers.key.first.as_slice(),headers.key.ordinal],|row|row.get(0)).map_err(database_error)?;
        let expected_count=if headers.failure.is_some(){2}else{1};
        if count!=expected_count{return Err(Failure::CustodyConflict);}
        if let Some(failure)=&headers.failure{
            if !same_header(&read_header(&transaction,&headers.key,1)?,failure){return Err(Failure::CustodyConflict);}
            payload_pass(&transaction,&headers.key,failure,false)?;
        }
        payload_pass(&transaction,&headers.key,&headers.intent,false)?;
        let c01=payload_pass(&transaction,&headers.key,&headers.intent,true)?;
        let failure=headers.failure.as_ref().map(|failure|payload_pass(&transaction,&headers.key,failure,true)).transpose()?;
        transaction.commit().map_err(database_error)?;drop(db);proof.scope.verify_current(self)?;
        Ok((c01,failure))
    }
}
