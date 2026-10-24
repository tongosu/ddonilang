//! Exact R4.3 SQL profile. Explicit fresh provisioning, never a migration.
//! Source custody/admission and first-world authority are separate owners.
//! SourceRevision user_version=2 is a private product discriminator; it is
//! never a replacement for exact DDL checks or the nine-method capability.
//! Contract SQL_SHA256: b49c2b57f3bc8d65cbdf3ac885bb8c4974115a3fccce2ac922734b405de4d44f

pub(super) const PAYLOAD: &str = r#"CREATE TABLE vc04_prepared_payloads (
 payload_kind INTEGER NOT NULL CHECK(payload_kind IN (0,1)),
 payload_key BLOB NOT NULL CHECK(length(payload_key)=32),
 transaction_digest BLOB UNIQUE CHECK(transaction_digest IS NULL OR length(transaction_digest)=32),
 project_identity BLOB NOT NULL CHECK(length(project_identity) BETWEEN 1 AND 4096),
 expected_parent_history_revision_identity BLOB CHECK(expected_parent_history_revision_identity IS NULL OR length(expected_parent_history_revision_identity) BETWEEN 1 AND 4096),
 new_history_revision_identity BLOB NOT NULL CHECK(length(new_history_revision_identity) BETWEEN 1 AND 4096),
 execution_profile_identity BLOB NOT NULL CHECK(length(execution_profile_identity) BETWEEN 1 AND 4096),
 project_execution_snapshot_identity BLOB NOT NULL CHECK(length(project_execution_snapshot_identity)=32),
 canonical_snapshot_v2_bytes BLOB,
 snapshot_v2_byte_length_u64 INTEGER,
 snapshot_v2_sha256 BLOB,
 project_payload_bytes BLOB,
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 revision_envelope_bytes BLOB NOT NULL CHECK(length(revision_envelope_bytes) BETWEEN 1 AND 1000000),
 PRIMARY KEY(payload_kind,payload_key),
 UNIQUE(project_identity,new_history_revision_identity,revision_integrity_sha256,payload_kind,payload_key),
 CHECK((payload_kind=0 AND transaction_digest IS NULL AND payload_key=revision_integrity_sha256
   AND canonical_snapshot_v2_bytes IS NULL AND snapshot_v2_byte_length_u64 IS NULL AND snapshot_v2_sha256 IS NULL
   AND project_payload_bytes IS NOT NULL AND length(project_payload_bytes) BETWEEN 1 AND 2000000)
 OR (payload_kind=1 AND transaction_digest IS NOT NULL AND payload_key=transaction_digest
   AND project_payload_bytes IS NULL AND canonical_snapshot_v2_bytes IS NOT NULL
   AND length(canonical_snapshot_v2_bytes) BETWEEN 1 AND 2000000
   AND snapshot_v2_byte_length_u64 IS NOT NULL AND snapshot_v2_byte_length_u64=length(canonical_snapshot_v2_bytes)
   AND snapshot_v2_sha256 IS NOT NULL AND length(snapshot_v2_sha256)=32))
) STRICT"#;

pub(super) const REVISIONS: &str = r#"CREATE TABLE vc04_project_revisions (
 project_identity BLOB NOT NULL CHECK(length(project_identity) BETWEEN 1 AND 4096),
 history_revision_identity BLOB NOT NULL CHECK(length(history_revision_identity) BETWEEN 1 AND 4096),
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 payload_kind INTEGER NOT NULL CHECK(payload_kind IN (0,1)),
 payload_key BLOB NOT NULL CHECK(length(payload_key)=32),
 PRIMARY KEY(project_identity,history_revision_identity),
 UNIQUE(project_identity,history_revision_identity,revision_integrity_sha256,payload_kind,payload_key),
 UNIQUE(payload_kind,payload_key),
 FOREIGN KEY(project_identity,history_revision_identity,revision_integrity_sha256,payload_kind,payload_key)
 REFERENCES vc04_prepared_payloads(project_identity,new_history_revision_identity,revision_integrity_sha256,payload_kind,payload_key)
) STRICT"#;

pub(super) const HEADS: &str = r#"CREATE TABLE vc04_project_heads (
 project_identity BLOB PRIMARY KEY CHECK(length(project_identity) BETWEEN 1 AND 4096),
 history_revision_identity BLOB NOT NULL CHECK(length(history_revision_identity) BETWEEN 1 AND 4096),
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 payload_kind INTEGER NOT NULL CHECK(payload_kind IN (0,1)),
 payload_key BLOB NOT NULL CHECK(length(payload_key)=32),
 FOREIGN KEY(project_identity,history_revision_identity,revision_integrity_sha256,payload_kind,payload_key)
 REFERENCES vc04_project_revisions(project_identity,history_revision_identity,revision_integrity_sha256,payload_kind,payload_key)
) STRICT"#;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn database()->Connection {
        let db=Connection::open_in_memory().unwrap();
        db.execute_batch(&format!("PRAGMA foreign_keys=ON; {PAYLOAD}; {REVISIONS}; {HEADS};")).unwrap();
        db
    }
    fn insert_source(db:&Connection) {
        // Mechanical bytes only, not an admitted source artifact or receipt.
        db.execute_batch("INSERT INTO vc04_prepared_payloads VALUES
          (0,zeroblob(32),NULL,x'70',NULL,x'73',x'65',zeroblob(32),NULL,NULL,NULL,x'01',zeroblob(32),x'02');").unwrap();
    }
    #[test]
    fn vc04_source_sql_nullable_roles_fail_closed() {
        let db=database();insert_source(&db);
        for sql in [
            "UPDATE vc04_prepared_payloads SET transaction_digest=zeroblob(32)",
            "UPDATE vc04_prepared_payloads SET canonical_snapshot_v2_bytes=x'01'",
            "UPDATE vc04_prepared_payloads SET project_payload_bytes=x''",
            "UPDATE vc04_prepared_payloads SET payload_kind=2",
            "UPDATE vc04_prepared_payloads SET payload_kind=1,transaction_digest=zeroblob(32),project_payload_bytes=NULL",
        ] {assert!(db.execute_batch(sql).is_err(),"{sql}");}
        assert_eq!(db.query_row("SELECT count(*) FROM vc04_prepared_payloads WHERE payload_kind=0 AND transaction_digest IS NULL",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    }
    #[test]
    fn vc04_source_sql_same_head_rejects_orphan_and_wrong_kind() {
        let db=database();insert_source(&db);
        assert!(db.execute_batch("INSERT INTO vc04_project_heads VALUES(x'70',x'73',zeroblob(32),0,zeroblob(32));").is_err());
        db.execute_batch("INSERT INTO vc04_project_revisions VALUES(x'70',x'73',zeroblob(32),0,zeroblob(32));
          INSERT INTO vc04_project_heads VALUES(x'70',x'73',zeroblob(32),0,zeroblob(32));").unwrap();
        assert!(db.execute_batch("UPDATE vc04_project_heads SET payload_kind=1").is_err());
        assert_eq!(db.query_row("SELECT payload_kind FROM vc04_project_heads",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
}
