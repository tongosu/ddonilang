//! Cold, current Native transport admission. This is not a VM restore permit,
//! a normal-end proof or ReplayVerified; typed replay still checks the content.
use super::*;
use sha2::{Digest,Sha256};
use crate::replay_evidence::{ReplayBundleCandidateV1,ReplayLifecycleCandidateV1};

struct NativeReplayArchiveTransportV1 {
    proof:VerifiedExecutionFirstCheckpointV1,
    lifecycle:Vec<u8>,bundle:Vec<u8>,
    kind:ReplayArchiveReadKindV1,
}
impl NativeReplayArchiveTransportV1 {
    fn checked_lifecycle(&self,adapter:&SqliteAdapterV1)->Result<&[u8],Failure>{
        self.revalidate(adapter)?;Ok(&self.lifecycle)
    }
    fn revalidate(&self,adapter:&SqliteAdapterV1)->Result<(),Failure>{
        if self.proof.read_lifecycle_for(adapter,self.kind)?!=self.lifecycle{return Err(Failure::CustodyConflict);}
        Ok(())
    }
}
pub(crate) struct NativeReplayProgressTransportV1 {inner:NativeReplayArchiveTransportV1}
impl NativeReplayProgressTransportV1 {
    pub(crate) fn bundle(&self)->&[u8]{&self.inner.bundle}
    pub(crate) fn checked_lifecycle(&self,adapter:&SqliteAdapterV1)->Result<&[u8],Failure>{self.inner.checked_lifecycle(adapter)}
    pub(crate) fn revalidate(&self,adapter:&SqliteAdapterV1)->Result<(),Failure>{self.inner.revalidate(adapter)}
}
pub(crate) struct NativeReplayFinishedTransportV1 {inner:NativeReplayArchiveTransportV1}
impl NativeReplayFinishedTransportV1 {
    pub(crate) fn bundle(&self)->&[u8]{&self.inner.bundle}
    pub(crate) fn revalidate(&self,adapter:&SqliteAdapterV1)->Result<(),Failure>{self.inner.revalidate(adapter)}
}
/// Only the explicit new-record path consumes this terminal transport. It is
/// neither Running progress nor a successful normal receipt replay.
pub(crate) struct NativeReplayTerminalTransportV1 {inner:NativeReplayArchiveTransportV1}
impl NativeReplayTerminalTransportV1 {
    pub(crate) fn bundle(&self)->&[u8]{&self.inner.bundle}
    pub(crate) fn anchor_count(&self)->u64{self.inner.proof.anchor_madi()}
    pub(crate) fn checked_lifecycle(&self,adapter:&SqliteAdapterV1)->Result<&[u8],Failure>{self.inner.checked_lifecycle(adapter)}
    pub(crate) fn scope_identity(&self,adapter:&SqliteAdapterV1)->Result<([u8;32],[u8;32],[u8;32],u64),Failure>{
        self.inner.revalidate(adapter)?;self.inner.proof.scope.identity_fields(adapter)
    }
    pub(crate) fn revalidate(&self,adapter:&SqliteAdapterV1)->Result<(),Failure>{self.inner.revalidate(adapter)}
    pub(crate) fn into_plan_parts(self,adapter:&SqliteAdapterV1)
        ->Result<(ReplayArchiveRootScopeV1,Vec<u8>,Vec<u8>),Failure>{
        self.inner.revalidate(adapter)?;
        let (scope,lifecycle)=self.inner.proof.into_new_record_plan_scope(adapter)?;
        Ok((scope,lifecycle,self.inner.bundle))
    }
}
struct Row {
    ordinal:u64,epoch:u64,checkpoint:[u8;32],state:u8,previous:[u8;32],
    bundle_hash:[u8;32],length:u64,chunks:u64,cipher:Vec<u8>,digest:[u8;32],
}
fn invalid()->rusqlite::Error{rusqlite::Error::InvalidQuery}
fn fixed(row:&rusqlite::Row<'_>,index:usize)->rusqlite::Result<[u8;32]>{
    row.get_ref(index)?.as_blob()?.try_into().map_err(|_|invalid())
}
fn number(row:&rusqlite::Row<'_>,index:usize)->rusqlite::Result<u64>{
    u64::try_from(row.get::<_,i64>(index)?).map_err(|_|invalid())
}
fn decode(row:&rusqlite::Row<'_>)->rusqlite::Result<Row>{
    let state=number(row,3)?;let length=number(row,6)?;let chunks=number(row,7)?;
    let cipher=row.get_ref(8)?.as_blob()?;
    if state>3 || length==0 || length>134217728 || chunks==0 || chunks>135
        || chunks!=(length+999999)/1000000 || cipher.is_empty() || cipher.len()>16384 {return Err(invalid());}
    Ok(Row{ordinal:number(row,0)?,epoch:number(row,1)?,checkpoint:fixed(row,2)?,state:state as u8,
        previous:fixed(row,4)?,bundle_hash:fixed(row,5)?,length,chunks,cipher:cipher.to_vec(),digest:fixed(row,9)?})
}
fn load_bundle(db:&Connection,first:&[u8;32],row:&Row)->Result<Vec<u8>,Failure>{
    let mut bytes=Vec::new();bytes.try_reserve_exact(row.length as usize).map_err(|_|Failure::LimitExceeded)?;
    let mut statement=db.prepare("SELECT chunk_ordinal,chunk_bytes FROM wo23_replay_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 ORDER BY chunk_ordinal").map_err(database_error)?;
    let mut rows=statement.query(params![first.as_slice(),row.ordinal as i64]).map_err(database_error)?;
    let mut ordinal=0u64;
    while let Some(part)=rows.next().map_err(database_error)?{
        if number(part,0).map_err(database_error)?!=ordinal || ordinal>=row.chunks{return Err(Failure::CorruptRecoveryRecord);}
        let chunk=part.get_ref(1).map_err(database_error)?.as_blob().map_err(|_|Failure::CorruptRecoveryRecord)?;
        let expected=if ordinal+1==row.chunks {row.length-(row.chunks-1)*1000000}else{1000000};
        if chunk.len() as u64!=expected{return Err(Failure::CorruptRecoveryRecord);}
        bytes.extend_from_slice(chunk);ordinal=ordinal.checked_add(1).ok_or(Failure::LimitExceeded)?;
    }
    if ordinal!=row.chunks || bytes.len() as u64!=row.length || <[u8;32]>::from(Sha256::digest(&bytes))!=row.bundle_hash {
        return Err(Failure::CorruptRecoveryRecord);
    }Ok(bytes)
}
impl SqliteAdapterV1 {
    pub(crate) fn load_current_replay_progress_transport(&self,proof:VerifiedExecutionFirstCheckpointV1)
        ->Result<NativeReplayProgressTransportV1,Failure>{
        self.load_replay_archive_transport(proof,ReplayArchiveReadKindV1::Running).map(|inner|NativeReplayProgressTransportV1{inner})
    }
    pub(crate) fn load_current_replay_finished_transport(&self,proof:VerifiedExecutionFirstCheckpointV1)
        ->Result<NativeReplayFinishedTransportV1,Failure>{
        self.load_replay_archive_transport(proof,ReplayArchiveReadKindV1::Finished).map(|inner|NativeReplayFinishedTransportV1{inner})
    }
    pub(crate) fn load_current_replay_terminal_transport(&self,proof:VerifiedExecutionFirstCheckpointV1)
        ->Result<NativeReplayTerminalTransportV1,Failure>{
        self.load_replay_archive_transport(proof,ReplayArchiveReadKindV1::TerminalForNewRecord).map(|inner|NativeReplayTerminalTransportV1{inner})
    }
    fn load_replay_archive_transport(&self,proof:VerifiedExecutionFirstCheckpointV1,kind:ReplayArchiveReadKindV1)
        ->Result<NativeReplayArchiveTransportV1,Failure>{
        let lifecycle=proof.read_lifecycle_for(self,kind)?;
        let bundle=self.check_replay_archive_candidate(&proof,kind,&lifecycle)?;
        if proof.read_lifecycle_for(self,kind)?!=lifecycle{return Err(Failure::CustodyConflict);}
        Ok(NativeReplayArchiveTransportV1{proof,lifecycle,kind,bundle})
    }

    /// Full archive/anchor validation only. Returning bytes does not issue a
    /// Ready, restore, execution or recovery permit. Normal readers still
    /// require protected Ready before and after this call. R2's separate
    /// Native owner may check its exact Ready candidate before a repair.
    pub(in crate::snapshot_v2_durable_transaction::native) fn check_replay_archive_candidate(
        &self,proof:&VerifiedExecutionFirstCheckpointV1,kind:ReplayArchiveReadKindV1,lifecycle:&[u8])
        ->Result<Vec<u8>,Failure>{
        let latest=ReplayLifecycleCandidateV1::parse(lifecycle).map_err(|_|Failure::CorruptRecoveryRecord)?;
        let (_,nonce,fingerprint,generation)=proof.scope.identity_fields(self)?;
        let state=latest.payload(9).and_then(|v|v.first()).copied().ok_or(Failure::CorruptRecoveryRecord)?;
        let count=u64::from_be_bytes(latest.payload(15).ok_or(Failure::CorruptRecoveryRecord)?.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?);
        let receipt_hash:[u8;32]=Sha256::digest(&proof.receipt).into();
        if latest.payload(1)!=Some(nonce.as_slice()) || latest.payload(2)!=Some(fingerprint.as_slice())
            || latest.payload(3)!=Some(generation.to_be_bytes().as_slice()) || latest.payload(4)!=Some(proof.pesi.as_slice())
            || latest.payload(5)!=Some(proof.first_tx.as_slice()) || latest.payload(8)!=Some(&[1][..])
            || !kind.accepts_state(state) || latest.payload(10)!=Some(proof.checkpoint_tx.as_slice())
            || latest.payload(11)!=Some(receipt_hash.as_slice()) || !kind.accepts_count(state,count,proof.anchor_madi())
            || !latest.payload(17).is_some_and(|value|kind.accepts_end(state,value[0])){
            return Err(Failure::CustodyConflict);
        }
        let latest_ordinal=u64::from_be_bytes(latest.payload(7).ok_or(Failure::CorruptRecoveryRecord)?.try_into().map_err(|_|Failure::CorruptRecoveryRecord)?);
        let db=self.connect()?;db.execute_batch("BEGIN").map_err(database_error)?;
        let count:i64=db.query_row("SELECT count(*) FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1",params![proof.first_tx.as_slice()],|r|r.get(0)).map_err(database_error)?;
        if u64::try_from(count).ok()!=latest_ordinal.checked_add(1){return Err(Failure::CustodyConflict);}
        let mut statement=db.prepare("SELECT control_ordinal,recording_epoch,checkpoint_tx,recording_state,previous_row_sha256,bundle_sha256,bundle_byte_length,chunk_count,custody_ciphertext,row_sha256 FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1 ORDER BY control_ordinal").map_err(database_error)?;
        let rows=statement.query_map(params![proof.first_tx.as_slice()],decode).map_err(database_error)?;
        let mut ordinal=0u64;let mut previous=[0;32];let mut latest_bundle=None;
        for row in rows {
            let row=row.map_err(database_error)?;
            if row.ordinal!=ordinal || row.previous!=previous{return Err(Failure::CustodyConflict);}
            let plain=proof.scope.unprotect_replay_custody(self,&row.cipher)?;
            let custody=crate::replay_evidence::ReplayCustodyCandidateV1::parse(&plain).map_err(|_|Failure::CorruptRecoveryRecord)?;
            let plain_hash:[u8;32]=Sha256::digest(&plain).into();
            let canonical=crate::replay_evidence::control_row_canonical_bytes([&proof.first_tx,&row.ordinal.to_be_bytes(),
                &row.epoch.to_be_bytes(),&row.checkpoint,&[row.state],&row.previous,&row.bundle_hash,
                &row.length.to_be_bytes(),&row.chunks.to_be_bytes(),&plain_hash]).map_err(|_|Failure::CorruptRecoveryRecord)?;
            if <[u8;32]>::from(Sha256::digest(&canonical))!=row.digest || custody.payload(4)!=Some(row.checkpoint.as_slice())
                || custody.payload(5)!=Some(row.bundle_hash.as_slice()) || custody.payload(6)!=Some(proof.pesi.as_slice()) {
                return Err(Failure::CustodyConflict);
            }
            let bundle=load_bundle(&db,&proof.first_tx,&row)?;
            let parsed=ReplayBundleCandidateV1::parse(&bundle).map_err(|_|Failure::CorruptRecoveryRecord)?;
            let mut control=vec![row.state];control.extend_from_slice(&row.epoch.to_be_bytes());control.extend_from_slice(&row.ordinal.to_be_bytes());
            control.extend_from_slice(&proof.first_tx);control.extend_from_slice(&row.checkpoint);control.extend_from_slice(&row.previous);
            let result=parsed.payload(6).ok_or(Failure::CorruptRecoveryRecord)?;
            let result_hash:[u8;32]=Sha256::digest(result).into();
            if parsed.payload(11)!=Some(control.as_slice()) || parsed.payload(7)!=Some(result_hash.as_slice())
                || custody.payload(7)!=Some(result_hash.as_slice()) {return Err(Failure::CustodyConflict);}
            if ordinal==latest_ordinal {
                let seed=parsed.payload(2).ok_or(Failure::CorruptRecoveryRecord)?;
                crate::replay_evidence::ReplaySeedCandidateV1::parse(seed).map_err(|_|Failure::CorruptRecoveryRecord)?;
                let seed_hash:[u8;32]=Sha256::digest(seed).into();
                let count=&parsed.payload(5).ok_or(Failure::CorruptRecoveryRecord)?[8..];
                let mut receipt=vec![1];receipt.extend_from_slice(&proof.receipt);
                let mut transaction=vec![1];transaction.extend_from_slice(&proof.checkpoint_tx);
                if row.checkpoint!=proof.checkpoint_tx || !kind.accepts_state(row.state) || latest.payload(6)!=Some(row.epoch.to_be_bytes().as_slice())
                    || latest.payload(12)!=Some(row.digest.as_slice()) || latest.payload(13)!=Some(row.previous.as_slice())
                    || latest.payload(14)!=Some(seed_hash.as_slice()) || latest.payload(15)!=Some(count)
                    || latest.payload(16)!=Some(result_hash.as_slice()) || !custody.payload(8).is_some_and(|value|kind.accepts_end(row.state,value[0]))
                    || custody.payload(8)!=latest.payload(17)
                    || parsed.payload(9)!=Some(receipt.as_slice()) || parsed.payload(10)!=Some(transaction.as_slice()) {
                    return Err(Failure::CustodyConflict);
                }latest_bundle=Some(bundle);
            }
            previous=row.digest;ordinal=ordinal.checked_add(1).ok_or(Failure::LimitExceeded)?;
        }
        drop(statement);db.execute_batch("COMMIT").map_err(database_error)?;drop(db);
        proof.scope.verify_current(self)?;
        latest_bundle.ok_or(Failure::CorruptRecoveryRecord)
    }
}
