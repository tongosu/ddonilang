//! Exact R1.5 optional child schema; existing tables and user_version stay put.
use super::*;
#[cfg(windows)]
mod reader;
#[cfg(windows)]
pub(crate) use reader::{NativeReplayProgressTransportV1,NativeReplayFinishedTransportV1,NativeReplayTerminalTransportV1};
pub(super) const CONTROLS_NAME:&str="wo23_replay_controls_v1";
pub(super) const CHUNKS_NAME:&str="wo23_replay_chunks_v1";
pub(super) const CONTROLS:&str="CREATE TABLE wo23_replay_controls_v1 (
 execution_first_tx BLOB NOT NULL CHECK(typeof(execution_first_tx)='blob' AND length(execution_first_tx)=32),
 control_ordinal INTEGER NOT NULL CHECK(typeof(control_ordinal)='integer' AND control_ordinal>=0),
 recording_epoch INTEGER NOT NULL CHECK(typeof(recording_epoch)='integer' AND recording_epoch>=0),
 checkpoint_tx BLOB NOT NULL CHECK(typeof(checkpoint_tx)='blob' AND length(checkpoint_tx)=32),
 recording_state INTEGER NOT NULL CHECK(recording_state BETWEEN 0 AND 3),
 previous_row_sha256 BLOB NOT NULL CHECK(typeof(previous_row_sha256)='blob' AND length(previous_row_sha256)=32),
 bundle_sha256 BLOB NOT NULL CHECK(typeof(bundle_sha256)='blob' AND length(bundle_sha256)=32),
 bundle_byte_length INTEGER NOT NULL CHECK(typeof(bundle_byte_length)='integer' AND bundle_byte_length>0 AND bundle_byte_length<=134217728),
 chunk_count INTEGER NOT NULL CHECK(typeof(chunk_count)='integer' AND chunk_count>0 AND chunk_count<=135),
 custody_ciphertext BLOB NOT NULL CHECK(typeof(custody_ciphertext)='blob' AND length(custody_ciphertext)>0 AND length(custody_ciphertext)<=16384),
 row_sha256 BLOB NOT NULL CHECK(typeof(row_sha256)='blob' AND length(row_sha256)=32),
 PRIMARY KEY(execution_first_tx,control_ordinal)
) WITHOUT ROWID";
pub(super) const CHUNKS:&str="CREATE TABLE wo23_replay_chunks_v1 (
 execution_first_tx BLOB NOT NULL CHECK(typeof(execution_first_tx)='blob' AND length(execution_first_tx)=32),
 control_ordinal INTEGER NOT NULL CHECK(typeof(control_ordinal)='integer' AND control_ordinal>=0),
 chunk_ordinal INTEGER NOT NULL CHECK(typeof(chunk_ordinal)='integer' AND chunk_ordinal>=0 AND chunk_ordinal<135),
 chunk_bytes BLOB NOT NULL CHECK(typeof(chunk_bytes)='blob' AND length(chunk_bytes)>0 AND length(chunk_bytes)<=1000000),
 PRIMARY KEY(execution_first_tx,control_ordinal,chunk_ordinal),
 FOREIGN KEY(execution_first_tx,control_ordinal) REFERENCES wo23_replay_controls_v1(execution_first_tx,control_ordinal)
) WITHOUT ROWID";
pub(super) fn extend_expected(rows:&[(String,String)],profile:NativeSchemaProfile,
    expected:&mut Vec<(&'static str,&'static str)>)->Result<(),Failure> {
    if rows.iter().any(|(name,_)|name==CONTROLS_NAME || name==CHUNKS_NAME) {
        if !profile.has_custody(){return Err(Failure::UnsupportedDurability);}
        expected.extend([(CONTROLS_NAME,CONTROLS),(CHUNKS_NAME,CHUNKS)]);
    }
    expected.sort_by_key(|(name,_)|*name);Ok(())
}
#[cfg(windows)]
impl SqliteAdapterV1 {
    pub(crate) fn commit_replay_metadata(&self,plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,
        row:&crate::replay_evidence::PlannedReplayControlRowV1)->Result<(),String>{
        let failed=|stage:&str,cause:Failure|format!("E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: 기록 부속 거래 {stage} ({cause:?})");
        plan.scope.verify_current(self).map_err(|e|failed("Root 확인",e))?;
        if plan.previous_lifecycle.is_none() || plan.control_ordinal==0 {
            return Err("E_REPLAY_METADATA_PLAN: 실제 기존 checkpoint의 종료 제어만 허용합니다".into());
        }
        match (&plan.reopen,plan.recording_state){
            (Some(proof),0)=>proof.check_plan(plan,self)?,
            (None,1..=3)=>{},
            _=>return Err("E_REPLAY_METADATA_PLAN: 새 기록은 실제 정상 복원 결속이 필요합니다".into()),
        }
        let db=self.connect().map_err(|e|failed("열기",e))?;
        db.execute_batch("BEGIN IMMEDIATE").map_err(|e|failed("시작",database_error(e)))?;
        let result=(||->Result<(),Failure>{
            let b=&plan.binding;
            let head:i64=db.query_row("SELECT count(*) FROM vc04_project_heads WHERE project_identity=?1 AND history_revision_identity=?2 AND revision_integrity_sha256=?3 AND payload_kind=1 AND payload_key=?4",
                params![b.project,b.new_revision,b.revision_integrity.as_slice(),plan.checkpoint_tx.as_slice()],|r|r.get(0)).map_err(database_error)?;
            let count:i64=db.query_row("SELECT count(*) FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1",
                params![plan.first_tx.as_slice()],|r|r.get(0)).map_err(database_error)?;
            if head!=1 || u64::try_from(count).ok()!=Some(plan.control_ordinal){return Err(Failure::CustodyConflict);}
            insert_checkpoint_rows(&db,plan,row)
        })();
        if let Err(cause)=result{
            return match db.execute_batch("ROLLBACK"){
                Ok(())=>Err(failed("쓰기 거부",cause)),Err(rollback)=>Err(format!("{}; rollback={rollback}",failed("쓰기 불확실",cause))),
            };
        }
        db.execute_batch("COMMIT").map_err(|e|failed("확정 불확실",database_error(e)))?;drop(db);
        self.confirm_container_durable().map_err(|e|failed("내구 확인",e))?;
        self.check_replay_archive(plan,row).map_err(|e|failed("전체 재읽기",e))
    }
    #[cfg(test)]
    pub(crate) fn test_assert_first_replay_archive(&self,first_tx:&[u8;32],expected:&[u8]){
        let db=self.connect().unwrap();
        let (ordinal,state,length):(i64,i64,i64)=db.query_row("SELECT control_ordinal,recording_state,bundle_byte_length FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1",
            params![first_tx.as_slice()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!((ordinal,state,length),(0,0,expected.len() as i64));
        let mut statement=db.prepare("SELECT chunk_bytes FROM wo23_replay_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=0 ORDER BY chunk_ordinal").unwrap();
        let chunks=statement.query_map(params![first_tx.as_slice()],|r|r.get::<_,Vec<u8>>(0)).unwrap();
        let mut actual=Vec::new();for chunk in chunks{actual.extend(chunk.unwrap());}
        assert_eq!(actual,expected);
    }
    #[cfg(test)]
    pub(crate) fn test_flip_first_replay_byte(&self,first_tx:&[u8;32],row_digest:bool){
        // Isolated fixture mutation only. Product writes remain INSERT-only.
        let db=self.connect().unwrap();
        let (select,update)=if row_digest {
            ("SELECT row_sha256 FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1 AND control_ordinal=0",
             "UPDATE wo23_replay_controls_v1 SET row_sha256=?2 WHERE execution_first_tx=?1 AND control_ordinal=0")
        }else{
            ("SELECT chunk_bytes FROM wo23_replay_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=0 AND chunk_ordinal=0",
             "UPDATE wo23_replay_chunks_v1 SET chunk_bytes=?2 WHERE execution_first_tx=?1 AND control_ordinal=0 AND chunk_ordinal=0")
        };
        let mut bytes:Vec<u8>=db.query_row(select,params![first_tx.as_slice()],|r|r.get(0)).unwrap();
        bytes[0]^=1;
        assert_eq!(db.execute(update,params![first_tx.as_slice(),bytes]).unwrap(),1);
    }
    pub(crate) fn check_replay_archive(&self,plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,
        row:&crate::replay_evidence::PlannedReplayControlRowV1)->Result<(),Failure>{
        plan.scope.verify_current(self)?;
        let db=self.connect()?;db.execute_batch("BEGIN").map_err(database_error)?;
        check_exact_rows(&db,plan,row)?;
        db.execute_batch("COMMIT").map_err(database_error)?;
        let plain=plan.scope.unprotect_replay_custody(self,&row.custody_ciphertext)?;
        let parsed=crate::replay_evidence::ReplayCustodyCandidateV1::parse(&plain).map_err(|_|Failure::CorruptRecoveryRecord)?;
        if parsed.payload(4)!=Some(plan.checkpoint_tx.as_slice()) || parsed.payload(5)!=Some(plan.bundle_digest.as_slice())
            || parsed.payload(6)!=Some(plan.binding.pesi.as_slice()) || parsed.payload(7)!=Some(plan.result_digest.as_slice())
            || parsed.payload(8)!=Some([plan.end_owner_kind].as_slice()) {return Err(Failure::CustodyConflict);}
        plan.scope.verify_current(self)
    }
    pub(crate) fn provision_replay_archive(&mut self,scope:&ReplayArchiveRootScopeV1)->Result<(),Failure> {
        scope.verify_current(self)?;
        if !self.schema_profile.has_custody(){return Err(Failure::UnsupportedDurability);}
        let mut db=self.connect()?;
        let count:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name IN (?1,?2)",
            params![CONTROLS_NAME,CHUNKS_NAME],|row|row.get(0)).map_err(database_error)?;
        if count==2{return scope.verify_current(self);}
        if count!=0{return Err(Failure::UnsupportedDurability);}
        let tx=db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(database_error)?;
        tx.execute_batch(CONTROLS).map_err(database_error)?;tx.execute_batch(CHUNKS).map_err(database_error)?;
        tx.commit().map_err(database_error)?;drop(db);
        self.confirm_container_durable()?;scope.verify_current(self)
    }
}
#[cfg(windows)]
pub(super) fn insert_checkpoint_rows(db:&rusqlite::Connection,
    plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,
    row:&crate::replay_evidence::PlannedReplayControlRowV1)->Result<(),Failure>{
    // Caller owns the existing world/head IMMEDIATE transaction. Never begin
    // or commit a separate world transaction here; INSERT only, no replacement.
    if db.is_autocommit() || row.first_tx!=plan.first_tx || row.checkpoint_tx!=plan.checkpoint_tx || row.bundle_length!=plan.bundle.len() as u64
        || row.bundle_digest!=plan.bundle_digest || row.control_ordinal!=plan.control_ordinal || row.recording_epoch!=plan.recording_epoch
        || row.previous_row!=plan.previous_row || row.recording_state!=plan.recording_state
        || row.custody_ciphertext.is_empty() || row.custody_ciphertext.len()>16384 {
        return Err(Failure::CustodyConflict);
    }
    db.execute("INSERT INTO wo23_replay_controls_v1 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![
        row.first_tx.as_slice(),row.control_ordinal as i64,row.recording_epoch as i64,row.checkpoint_tx.as_slice(),
        row.recording_state as i64,row.previous_row.as_slice(),row.bundle_digest.as_slice(),row.bundle_length as i64,
        row.chunk_count as i64,&row.custody_ciphertext,row.row_digest.as_slice()]).map_err(database_error)?;
    for (ordinal,chunk) in plan.bundle.chunks(1000000).enumerate(){
        db.execute("INSERT INTO wo23_replay_chunks_v1 VALUES (?1,?2,?3,?4)",params![row.first_tx.as_slice(),
            row.control_ordinal as i64,ordinal as i64,chunk]).map_err(database_error)?;
    }
    check_exact_rows(db,plan,row)
}
#[cfg(windows)]
fn check_exact_rows(db:&rusqlite::Connection,plan:&crate::replay_evidence::ReplayCheckpointPlanDataV1,
    row:&crate::replay_evidence::PlannedReplayControlRowV1)->Result<(),Failure>{
    // SQLite performs fixed/bounded comparisons without decoding untrusted
    // blobs into Rust buffers. Unique PK plus exact count rejects extra chunks.
    let count:i64=db.query_row("SELECT count(*) FROM wo23_replay_controls_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND recording_epoch=?3 AND checkpoint_tx=?4 AND recording_state=?5 AND previous_row_sha256=?6 AND bundle_sha256=?7 AND bundle_byte_length=?8 AND chunk_count=?9 AND custody_ciphertext=?10 AND row_sha256=?11",params![
        row.first_tx.as_slice(),row.control_ordinal as i64,row.recording_epoch as i64,row.checkpoint_tx.as_slice(),
        row.recording_state as i64,row.previous_row.as_slice(),row.bundle_digest.as_slice(),row.bundle_length as i64,
        row.chunk_count as i64,&row.custody_ciphertext,row.row_digest.as_slice()],|r|r.get(0)).map_err(database_error)?;
    if count!=1{return Err(Failure::RereadMismatch);}
    let count:i64=db.query_row("SELECT count(*) FROM wo23_replay_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2",
        params![row.first_tx.as_slice(),row.control_ordinal as i64],|r|r.get(0)).map_err(database_error)?;
    if count!=row.chunk_count as i64 || count!=plan.bundle.chunks(1000000).len() as i64{return Err(Failure::RereadMismatch);}
    for (ordinal,chunk) in plan.bundle.chunks(1000000).enumerate(){
        let exact:i64=db.query_row("SELECT count(*) FROM wo23_replay_chunks_v1 WHERE execution_first_tx=?1 AND control_ordinal=?2 AND chunk_ordinal=?3 AND chunk_bytes=?4",
            params![row.first_tx.as_slice(),row.control_ordinal as i64,ordinal as i64,chunk],|r|r.get(0)).map_err(database_error)?;
        if exact!=1{return Err(Failure::RereadMismatch);}
    }Ok(())
}
