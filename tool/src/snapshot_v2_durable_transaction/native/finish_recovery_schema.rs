//! WO25 R2 optional, explicit same-DB child. No boot/read-time provisioning.
use super::*;
pub(super) const RECORDS_NAME:&str="wo25_finish_recovery_records_v1";
pub(super) const CHUNKS_NAME:&str="wo25_finish_recovery_chunks_v1";
pub(super) const RECORDS:&str="CREATE TABLE wo25_finish_recovery_records_v1 (
 execution_first_tx BLOB NOT NULL CHECK(typeof(execution_first_tx)='blob' AND length(execution_first_tx)=32),
 control_ordinal INTEGER NOT NULL CHECK(typeof(control_ordinal)='integer' AND control_ordinal>=0),
 recording_epoch INTEGER NOT NULL CHECK(typeof(recording_epoch)='integer' AND recording_epoch>=0),
 evidence_ordinal INTEGER NOT NULL CHECK(typeof(evidence_ordinal)='integer' AND evidence_ordinal IN (0,1)),
 kind INTEGER NOT NULL CHECK(typeof(kind)='integer' AND kind=evidence_ordinal),
 header_bytes BLOB NOT NULL CHECK(typeof(header_bytes)='blob' AND length(header_bytes)>0 AND length(header_bytes)<=131072),
 payload_length INTEGER NOT NULL CHECK(typeof(payload_length)='integer' AND payload_length>0 AND payload_length<=536870912),
 chunk_count INTEGER NOT NULL CHECK(typeof(chunk_count)='integer' AND chunk_count>0 AND chunk_count<=537),
 previous_evidence_sha256 BLOB NOT NULL CHECK(typeof(previous_evidence_sha256)='blob' AND length(previous_evidence_sha256)=32),
 record_sha256 BLOB NOT NULL CHECK(typeof(record_sha256)='blob' AND length(record_sha256)=32),
 PRIMARY KEY(execution_first_tx,control_ordinal,evidence_ordinal)
) WITHOUT ROWID";
pub(super) const CHUNKS:&str="CREATE TABLE wo25_finish_recovery_chunks_v1 (
 execution_first_tx BLOB NOT NULL CHECK(typeof(execution_first_tx)='blob' AND length(execution_first_tx)=32),
 control_ordinal INTEGER NOT NULL CHECK(typeof(control_ordinal)='integer' AND control_ordinal>=0),
 evidence_ordinal INTEGER NOT NULL CHECK(typeof(evidence_ordinal)='integer' AND evidence_ordinal IN (0,1)),
 chunk_ordinal INTEGER NOT NULL CHECK(typeof(chunk_ordinal)='integer' AND chunk_ordinal>=0 AND chunk_ordinal<537),
 chunk_bytes BLOB NOT NULL CHECK(typeof(chunk_bytes)='blob' AND length(chunk_bytes)>0 AND length(chunk_bytes)<=1000000),
 PRIMARY KEY(execution_first_tx,control_ordinal,evidence_ordinal,chunk_ordinal),
 FOREIGN KEY(execution_first_tx,control_ordinal,evidence_ordinal) REFERENCES wo25_finish_recovery_records_v1(execution_first_tx,control_ordinal,evidence_ordinal)
) WITHOUT ROWID";

pub(super) fn extend_expected(rows:&[(String,String)],profile:NativeSchemaProfile,
    expected:&mut Vec<(&'static str,&'static str)>)->Result<(),Failure>{
    let records=rows.iter().any(|(name,_)|name==RECORDS_NAME);
    let chunks=rows.iter().any(|(name,_)|name==CHUNKS_NAME);
    if records!=chunks{return Err(Failure::UnsupportedDurability);}
    if records{
        if !profile.has_custody(){return Err(Failure::UnsupportedDurability);}
        expected.extend([(RECORDS_NAME,RECORDS),(CHUNKS_NAME,CHUNKS)]);
    }
    expected.sort_by_key(|(name,_)|*name);Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn wo25_finish_native_schema_read_never_repairs_a_partial_child_family(){
        // Actual SQLite adapter mechanics, not a Native source/root grant.
        // Only this isolated fixture performs explicit DDL mutation.
        let mut nonce=[0;16];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-wo25-finish-schema-{}",hex::encode(nonce)));
        std::fs::create_dir(&root).unwrap();
        let adapter=SqliteAdapterV1::provision_source_revision(&root).unwrap();
        let db=adapter.connect().unwrap();
        db.execute_batch(RECORDS).unwrap();
        assert!(matches!(adapter.connect(),Err(Failure::UnsupportedDurability)),"one table cannot be silently provisioned by a reader");
        let count:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name=?1",[CHUNKS_NAME],|row|row.get(0)).unwrap();
        assert_eq!(count,0,"failed open must not create the missing table");
        db.execute_batch(CHUNKS).unwrap();drop(db);
        assert!(adapter.connect().is_ok(),"both exact optional tables are recognized mechanically, not admitted as authority");
        drop(adapter);
    }
}

#[cfg(windows)]
impl SqliteAdapterV1{
    /// Only the explicit selected-profile finish producer calls this method.
    /// Ordinary open, source admission and recovery reads never call it.
    pub(crate) fn provision_finish_recovery(&mut self,scope:&ReplayArchiveRootScopeV1)->Result<(),Failure>{
        scope.verify_current(self)?;
        if !self.schema_profile.has_custody(){return Err(Failure::UnsupportedDurability);}
        let mut db=self.connect()?;
        let count:i64=db.query_row("SELECT count(*) FROM sqlite_schema WHERE name IN (?1,?2)",
            params![RECORDS_NAME,CHUNKS_NAME],|row|row.get(0)).map_err(database_error)?;
        if count==2{return scope.verify_current(self);}
        if count!=0{return Err(Failure::UnsupportedDurability);}
        let transaction=db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(database_error)?;
        transaction.execute_batch(RECORDS).map_err(database_error)?;
        transaction.execute_batch(CHUNKS).map_err(database_error)?;
        transaction.commit().map_err(database_error)?;drop(db);
        self.confirm_container_durable()?;
        // connect performs the exact schema/family/pragma check again.
        drop(self.connect()?);scope.verify_current(self)
    }
}
