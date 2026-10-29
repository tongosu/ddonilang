//! RD-671 mechanics only. No identity, receipt, recovery winner or world publication.
//! The shared owner must admit all supplied bytes before using these operations.

use super::{SnapshotV2DurabilityFailureV1 as Failure, TransactionBindingV1};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::path::{Path, PathBuf};
use std::time::Duration;

mod root_grant;
mod replay_archive_schema;
mod finish_recovery_schema;
#[cfg(windows)]
mod finish_recovery_store;
#[cfg(windows)]
pub(crate) use replay_archive_schema::{NativeReplayProgressTransportV1,NativeReplayFinishedTransportV1,NativeReplayTerminalTransportV1};
#[cfg(windows)]
pub(crate) use replay_archive_schema::CapturedReplayMetadataFailureV1;
#[cfg(windows)]
pub(crate) use root_grant::ReplayArchiveRootScopeV1;
#[cfg(windows)]
pub(crate) use root_grant::DurableFinishIntentWitnessV1;
#[cfg(windows)]
pub(crate) use root_grant::{NativeFinishRecoveryPermitV1,RecoveredNativeFinishV1};
#[cfg(all(test,windows))]
pub(crate) use root_grant::test_arm_finish_ready_set_failure;
#[cfg(all(test,windows))]
pub(crate) use root_grant::test_finish_lifecycle_cipher_digest;
#[cfg(windows)]
pub(crate) use root_grant::VerifiedExecutionFirstCheckpointV1;
#[cfg(windows)]
pub(crate) use root_grant::ProtectedReplayReadyV1;
#[cfg(windows)]
pub(crate) use root_grant::ReplayLifecycleWriteFailureV1;
#[cfg(windows)]
pub(crate) use root_grant::ReplayArchiveReadKindV1;
#[cfg(windows)]
pub(crate) use root_grant::{AuthoringHeadV1,AdmittedAuthoringBaseV1,PreparedAuthoringSourceV1,DurablySavedAuthoringSourceV1};
#[cfg(windows)]
pub(crate) use root_grant::{NewSourceRunPermitV1,RestartSourceWorldAttemptV1};
mod source_revision_schema;
pub(crate) mod source_revision;
mod source_chain;
#[cfg(all(windows,any(test,feature="beta2-crash-probe")))]
pub(crate) mod crash_probe;
#[cfg(windows)]
pub(crate) use root_grant::DurableAttemptStartV1;
#[cfg(windows)]
pub(crate) use root_grant::RetainedAttemptWitnessV1;
#[cfg(windows)]
pub(crate) use root_grant::AdmittedInitialAttemptV1;
#[cfg(windows)]
pub(crate) use root_grant::RecoveryAttemptStartV1;
#[cfg(windows)]
pub(crate) use root_grant::FirstSourceWorldPermitV2;
#[cfg(windows)]
pub(crate) use root_grant::SourceWorldAttemptStartV2;
#[cfg(windows)]
pub(crate) use root_grant::NextSourceWorldAttemptStartV1;
#[cfg(windows)]
mod source_world;
#[cfg(windows)]
pub(crate) use source_world::PreparedFirstSourceWorldV2;
#[cfg(windows)]
pub(crate) use source_world::RetainedFirstSourceWorldV2;
#[cfg(windows)]
pub(crate) use root_grant::ReadmittedFirstSourceWorldV2;
#[cfg(all(windows,test))]
pub(crate) use source_world::inject_fault as inject_source_world_fault;

// RD-670/672 transport ceiling also covers the largest legal SQL record:
// 2MB snapshot + 1MB envelope + four <=4096-byte locators, digests and headers.
// Native SQLite must enforce this before producing a row for Rust decoding.
const MAX_SQLITE_RECORD_BYTES: i32 = 3_030_005;

const PAYLOAD: &str = "CREATE TABLE vc04_prepared_payloads (
 transaction_digest BLOB PRIMARY KEY CHECK(length(transaction_digest)=32),
 project_identity BLOB NOT NULL CHECK(length(project_identity) BETWEEN 1 AND 4096),
 expected_parent_history_revision_identity BLOB CHECK(expected_parent_history_revision_identity IS NULL OR length(expected_parent_history_revision_identity) BETWEEN 1 AND 4096),
 new_history_revision_identity BLOB NOT NULL CHECK(length(new_history_revision_identity) BETWEEN 1 AND 4096),
 execution_profile_identity BLOB NOT NULL CHECK(length(execution_profile_identity) BETWEEN 1 AND 4096),
 project_execution_snapshot_identity BLOB NOT NULL CHECK(length(project_execution_snapshot_identity)=32),
 canonical_snapshot_v2_bytes BLOB NOT NULL CHECK(length(canonical_snapshot_v2_bytes) BETWEEN 1 AND 2000000),
 snapshot_v2_byte_length_u64 INTEGER NOT NULL CHECK(snapshot_v2_byte_length_u64>0 AND snapshot_v2_byte_length_u64=length(canonical_snapshot_v2_bytes)),
 snapshot_v2_sha256 BLOB NOT NULL CHECK(length(snapshot_v2_sha256)=32),
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 revision_envelope_bytes BLOB NOT NULL CHECK(length(revision_envelope_bytes) BETWEEN 1 AND 1000000)
) STRICT";
const REVISIONS: &str = "CREATE TABLE vc04_project_revisions (
 project_identity BLOB NOT NULL CHECK(length(project_identity) BETWEEN 1 AND 4096),
 history_revision_identity BLOB NOT NULL CHECK(length(history_revision_identity) BETWEEN 1 AND 4096),
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 transaction_digest BLOB NOT NULL UNIQUE CHECK(length(transaction_digest)=32),
 PRIMARY KEY(project_identity,history_revision_identity),
 UNIQUE(project_identity,history_revision_identity,revision_integrity_sha256,transaction_digest),
 FOREIGN KEY(transaction_digest) REFERENCES vc04_prepared_payloads(transaction_digest)
) STRICT";
const HEADS: &str = "CREATE TABLE vc04_project_heads (
 project_identity BLOB PRIMARY KEY CHECK(length(project_identity) BETWEEN 1 AND 4096),
 history_revision_identity BLOB NOT NULL CHECK(length(history_revision_identity) BETWEEN 1 AND 4096),
 revision_integrity_sha256 BLOB NOT NULL CHECK(length(revision_integrity_sha256)=32),
 transaction_digest BLOB NOT NULL CHECK(length(transaction_digest)=32),
 FOREIGN KEY(project_identity,history_revision_identity,revision_integrity_sha256,transaction_digest)
 REFERENCES vc04_project_revisions(project_identity,history_revision_identity,revision_integrity_sha256,transaction_digest)
) STRICT";

// R2 exact private child schema. Explicit provisioning only; not a migration.
const RECOVERY_CUSTODY:&str="CREATE TABLE vc04_recovery_custody_v1 (
  transaction_digest BLOB NOT NULL CHECK(typeof(transaction_digest)='blob' AND length(transaction_digest)=32),
  journal_ordinal INTEGER NOT NULL CHECK(typeof(journal_ordinal)='integer' AND journal_ordinal>=0 AND journal_ordinal<64),
  custody_version INTEGER NOT NULL CHECK(custody_version=1),
  previous_row_sha256 BLOB NOT NULL CHECK(typeof(previous_row_sha256)='blob' AND length(previous_row_sha256)=32),
  admitted_request_bytes BLOB NOT NULL CHECK(typeof(admitted_request_bytes)='blob' AND length(admitted_request_bytes)>0),
  recovery_record_bytes BLOB NOT NULL CHECK(typeof(recovery_record_bytes)='blob' AND length(recovery_record_bytes)>0),
  row_sha256 BLOB NOT NULL CHECK(typeof(row_sha256)='blob' AND length(row_sha256)=32),
  PRIMARY KEY(transaction_digest,journal_ordinal)
) WITHOUT ROWID";

#[derive(Clone,Copy,PartialEq,Eq)]
enum NativeSchemaProfile { Base, RecoveryCustody, SourceRevision }

impl NativeSchemaProfile {
    fn user_version(self)->i64 { if self==Self::SourceRevision {2}else{1} }
    fn source_tables(self)->[(&'static str,&'static str);3] {
        if self==Self::SourceRevision {
            [("vc04_prepared_payloads",source_revision_schema::PAYLOAD),
             ("vc04_project_heads",source_revision_schema::HEADS),
             ("vc04_project_revisions",source_revision_schema::REVISIONS)]
        }else{[("vc04_prepared_payloads",PAYLOAD),("vc04_project_heads",HEADS),("vc04_project_revisions",REVISIONS)]}
    }
    fn has_custody(self)->bool { self!=Self::Base }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StoredPayloadV1 {
    pub digest: [u8; 32],
    pub binding: TransactionBindingV1,
    pub snapshot: Vec<u8>,
    pub envelope: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StoredHeadV1 {
    pub project: Vec<u8>,
    pub revision: Vec<u8>,
    pub integrity: [u8; 32],
    pub digest: [u8; 32],
}

#[derive(Debug)]
pub(crate) struct RawChainV1 {
    pub payload: Option<StoredPayloadV1>,
    pub revision: Option<StoredHeadV1>,
    pub head: Option<StoredHeadV1>,
}

pub(crate) struct PreparedWriteV1 {
    connection: Option<Connection>,
    path: PathBuf,
    root_lease: Option<std::sync::Arc<std::fs::File>>,
    #[cfg(windows)]
    pinned_root: Option<std::sync::Arc<root_grant::PinnedNativeRootV1>>,
}
pub(crate) struct SqliteAdapterV1 {
    path: PathBuf,
    schema_profile: NativeSchemaProfile,
    // Live mechanical coordination only, never a persisted semantic identity.
    root_lease: Option<std::sync::Arc<std::fs::File>>,
    #[cfg(windows)]
    pinned_root: Option<std::sync::Arc<root_grant::PinnedNativeRootV1>>,
    #[cfg(windows)]
    fresh_container: std::cell::RefCell<Option<root_grant::FreshNativeContainerV1>>,
}

fn normalize(sql: &str) -> String {
    sql.split_whitespace().collect::<String>()
}
fn database_error(_: rusqlite::Error) -> Failure {
    Failure::RereadFailed
}

impl SqliteAdapterV1 {
    /// Explicit one-time provisioning. Never migrates or replaces an existing
    /// database. The caller supplies its already approved storage root.
    pub(crate) fn provision(root: &Path) -> Result<Self, Failure> {
        Self::provision_profile(root,NativeSchemaProfile::Base)
    }

    /// A fresh child-profile container only. Existing DBs are never upgraded.
    /// No transaction authority or Pending-start capability is issued here.
    pub(crate) fn provision_recovery_custody(root:&Path) -> Result<Self,Failure> {
        Self::provision_profile(root,NativeSchemaProfile::RecoveryCustody)
    }

    /// R4.3/R4.4: new empty DB only. This is mechanics, not a source-parent
    /// capability; INITIAL and world-parent guards remain unchanged.
    pub(crate) fn provision_source_revision(root:&Path)->Result<Self,Failure> {
        Self::provision_profile(root,NativeSchemaProfile::SourceRevision)
    }

    pub(crate) fn open_source_revision(root:&Path)->Result<Self,Failure> {
        Self::open_admitted_profile(root,NativeSchemaProfile::SourceRevision)
    }

    fn provision_profile(root:&Path,schema_profile:NativeSchemaProfile) -> Result<Self,Failure> {
        let (root, lease) = acquire_local_root(root)?;
        sync_directory(&root)?;
        let path = root.join("vc04_snapshot_v2.sqlite3");
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    Failure::ConflictingRetry
                } else {
                    Failure::WriteFailed
                }
            })?
            .sync_all()
            .map_err(|_| Failure::SyncFailed)?;
        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| Failure::WriteFailed)?;
        bound_sqlite_records(&connection)?;
        let mode: String = connection
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|_| Failure::WriteFailed)?;
        if mode != "wal" {
            return Err(Failure::UnsupportedDurability);
        }
        retain_wal(&connection)?;
        let child_schema=if schema_profile.has_custody() {format!("{RECOVERY_CUSTODY};")}else{String::new()};
        let tables=schema_profile.source_tables().iter().map(|(_,sql)|format!("{sql};")).collect::<String>();
        let version=schema_profile.user_version();
        connection
            .execute_batch(&format!(
                "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA read_uncommitted=OFF;
             BEGIN IMMEDIATE; {tables} {child_schema} PRAGMA user_version={version}; COMMIT;"
            ))
            .map_err(|_| Failure::WriteFailed)?;
        drop(connection);
        let mut adapter = Self {
            path,
            schema_profile,
            root_lease: Some(std::sync::Arc::new(lease)),
            #[cfg(windows)] pinned_root: None,
            #[cfg(windows)] fresh_container: std::cell::RefCell::new(None),
        };
        // Fresh connection + actual file/directory sync, not provisioning SQL
        // success or a configuration flag, is the capability gate.
        adapter.capability()?;
        #[cfg(windows)]
        { *adapter.fresh_container.get_mut() = Some(root_grant::FreshNativeContainerV1::capture_created(&adapter.path)?); }
        Ok(adapter)
    }

    pub(crate) fn open_admitted(root: &Path) -> Result<Self, Failure> {
        Self::open_admitted_profile(root,NativeSchemaProfile::Base)
    }

    pub(crate) fn open_recovery_custody(root:&Path) -> Result<Self,Failure> {
        Self::open_admitted_profile(root,NativeSchemaProfile::RecoveryCustody)
    }

    fn open_admitted_profile(root:&Path,schema_profile:NativeSchemaProfile) -> Result<Self,Failure> {
        let (root, lease) = acquire_local_root(root)?;
        let path = root.join("vc04_snapshot_v2.sqlite3");
        reject_reparse(&path)?;
        let adapter = Self {
            path,
            schema_profile,
            root_lease: Some(std::sync::Arc::new(lease)),
            #[cfg(windows)] pinned_root: None,
            #[cfg(windows)] fresh_container: std::cell::RefCell::new(None),
        };
        adapter.capability()?;
        Ok(adapter)
    }

    /// Does not provision, migrate, change journal mode or create missing files.
    /// A path is a locator only. Durable capability is a separate actual I/O gate.
    pub(crate) fn open_existing(path: &Path) -> Result<Self, Failure> {
        let adapter = Self {
            path: path.to_owned(),
            schema_profile:NativeSchemaProfile::Base,
            root_lease: None,
            #[cfg(windows)] pinned_root: None,
            #[cfg(windows)] fresh_container: std::cell::RefCell::new(None),
        };
        adapter.connect()?;
        Ok(adapter)
    }

    fn connect(&self) -> Result<Connection, Failure> {
        let connection = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| Failure::UnsupportedDurability)?;
        #[cfg(windows)]
        if let Some(pinned) = &self.pinned_root { pinned.verify_connection(&connection)?; }
        bound_sqlite_records(&connection)?;
        connection
            .busy_timeout(Duration::from_millis(250))
            .map_err(database_error)?;
        let mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(database_error)?;
        if mode != "wal" {
            return Err(Failure::UnsupportedDurability);
        }
        retain_wal(&connection)?;
        // These settings are connection-local. The persistent journal mode is never changed here.
        connection
            .execute_batch(
                "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA read_uncommitted=OFF;",
            )
            .map_err(database_error)?;
        for (name, expected) in [
            ("synchronous", 2),
            ("foreign_keys", 1),
            ("read_uncommitted", 0),
            ("user_version", self.schema_profile.user_version()),
        ] {
            let value: i64 = connection
                .query_row(&format!("PRAGMA {name}"), [], |row| row.get(0))
                .map_err(database_error)?;
            if value != expected {
                return Err(Failure::UnsupportedDurability);
            }
        }
        let mut statement = connection
            .prepare("SELECT name,sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(database_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?;
        let mut expected = self.schema_profile.source_tables().to_vec();
        if self.schema_profile.has_custody() {
            expected.push(("vc04_recovery_custody_v1",RECOVERY_CUSTODY));
            expected.sort_by_key(|(name,_)|*name);
        }
        replay_archive_schema::extend_expected(&rows,self.schema_profile,&mut expected)?;
        finish_recovery_schema::extend_expected(&rows,self.schema_profile,&mut expected)?;
        if rows.len() != expected.len()
            || rows
                .iter()
                .zip(expected)
                .any(|((name, sql), (target, ddl))| {
                    name != target || normalize(sql) != normalize(ddl)
                })
        {
            return Err(Failure::UnsupportedDurability);
        }
        drop(statement);
        Ok(connection)
    }

    #[cfg(windows)]
    fn pin_objects_for_grant(&mut self, registration: bool) -> Result<(), Failure> {
        // Mechanical pinning is not grant admission. Only an explicit host
        // registration may choose CREATE; cold callers must choose GET only.
        self.capability()?;
        let pinned = root_grant::PinnedNativeRootV1::capture(&self.path, registration)?;
        pinned.verify_connection(&self.connect()?)?;
        self.pinned_root = Some(std::sync::Arc::new(pinned));
        Ok(())
    }

    /// SQLite profile and actual root-sync support, not an adapter-provided bool.
    /// Unsupported directory flush is explicitly refused on the current platform.
    pub(crate) fn capability(&self) -> Result<&'static str, Failure> {
        if self.root_lease.is_none() {
            return Err(Failure::UnsupportedDurability);
        }
        reject_reparse(&self.path)?;
        self.connect()?;
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|_| Failure::UnsupportedDurability)?;
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or(Failure::UnsupportedDurability)?;
        for suffix in ["-wal", "-shm"] {
            let mut name = self.path.as_os_str().to_owned();
            name.push(suffix);
            reject_reparse(Path::new(&name))?;
        }
        sync_directory(parent)?;
        Ok("vc04.sqlite-durability.r1")
    }

    /// Called before the common registration issuer, with this adapter's
    /// exclusive OS root lease still held. A visible caller-supplied head is
    /// not sufficient: compare the complete actual SQLite head as well.
    pub(crate) fn verify_registration_expected_head(
        &mut self, project:&[u8], expected:Option<&StoredHeadV1>,
    )->Result<(),Failure> {
        self.capability()?;
        if project.len()!=32 || expected.is_some_and(|head|head.project!=project) {
            return Err(Failure::CustodyConflict);
        }
        let connection=self.connect()?;
        if self.schema_profile==NativeSchemaProfile::SourceRevision {
            // Initial source authoring only. A source head has no transaction
            // digest and must never be cast to the old world-head structure.
            if expected.is_some() {return Err(Failure::UnsupportedDurability);}
            #[cfg(windows)]
            self.fresh_container.borrow().as_ref().ok_or(Failure::CustodyConflict)?.verify_source_connection(&self.path,&connection)?;
            let count:i64=connection.query_row("SELECT count(*) FROM vc04_project_heads",[],|row|row.get(0)).map_err(database_error)?;
            return if count==0 {Ok(())}else{Err(Failure::StaleRevision)};
        }
        if read_head(&connection,"vc04_project_heads",project,None)?.as_ref()!=expected {
            return Err(Failure::StaleRevision);
        }
        Ok(())
    }

    pub(crate) fn verify_registration_parent_resources(
        &mut self,project:&[u8],expected:Option<&StoredHeadV1>,
        previous:Option<&crate::authored_project::registration::RegistrationDraft>,
    )->Result<(),Failure> {
        self.verify_registration_expected_head(project,expected)?;
        match (expected,previous) {
            (None,None)=>Ok(()),
            (Some(head),Some(draft))=>{
                let connection=self.connect()?;
                let payload=read_payload(&connection,&head.digest)?.ok_or(Failure::CustodyConflict)?;
                if payload.binding.project!=project||payload.binding.new_revision!=head.revision
                    ||payload.binding.revision_integrity!=head.integrity
                    ||payload.binding.transaction_digest()?!=head.digest {return Err(Failure::CustodyConflict);}
                let envelope=crate::project_revision_envelope::verify_rd631_canonical_envelope_bytes(&payload.envelope)?;
                crate::project_revision_envelope::verify_rd631_envelope_structure(&envelope)?;
                if crate::project_revision_envelope::verify_self_empty_digest(&envelope,"revision_integrity_sha256",Failure::CustodyConflict)?!=head.integrity{return Err(Failure::CustodyConflict);}
                let members=serde_json::Value::Array(draft.artifacts().iter().map(|blob|
                    serde_json::json!({"logical_path":blob.path(),"content_identity":blob.sha()})).collect());
                if envelope["declared_resource_members"]!=members{return Err(Failure::CustodyConflict);}
                Ok(())
            }
            // An existing head is not fresh-authoring permission when its
            // registered artifact was lost; an unsaved draft is not history.
            _=>Err(Failure::CustodyConflict),
        }
    }

    pub(crate) fn write_prepared(
        &self,
        payload: &StoredPayloadV1,
    ) -> Result<PreparedWriteV1, Failure> {
        // The base writer has no Pending-start proof or atomic row0. Refuse
        // the child schema rather than silently bypassing its owner fence.
        if self.schema_profile!=NativeSchemaProfile::Base {return Err(Failure::UnsupportedDurability);}
        self.write_prepared_rows(payload,None)
    }

    #[cfg(windows)]
    pub(super) fn write_initial_prepared_with_custody(&self,start:&DurableAttemptStartV1,
        source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        row:&super::recovery_custody::InitialCustodyRowV1)->Result<PreparedWriteV1,Failure> {
        if self.schema_profile!=NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
        start.check_current(self,source)?;
        row.check_request(start.request())?;
        self.write_prepared_rows(start.request().payload(),Some(row))
    }

    fn write_prepared_rows(&self,payload:&StoredPayloadV1,
        custody:Option<&super::recovery_custody::InitialCustodyRowV1>)->Result<PreparedWriteV1,Failure> {
        let _observation = crate::storage_observation::Span::start("sqlite.write_prepared_total");
        #[cfg(windows)]
        { self.fresh_container.try_borrow_mut().map_err(|_| Failure::CustodyConflict)?.take(); }
        // Reuse RD-670/672 transport ceilings before even opening a database.
        if payload.snapshot.len() > 2_000_000
            || payload.envelope.len() > 1_000_000
            || [
                &payload.binding.project,
                &payload.binding.expected_parent,
                &payload.binding.new_revision,
                &payload.binding.execution_profile,
            ]
            .iter()
            .any(|bytes| bytes.len() > 4096)
        {
            return Err(Failure::LimitExceeded);
        }
        if payload.snapshot.is_empty()
            || payload.envelope.is_empty()
            || payload.binding.snapshot_length != payload.snapshot.len() as u64
        {
            return Err(Failure::InvalidInput);
        }
        let length =
            i64::try_from(payload.binding.snapshot_length).map_err(|_| Failure::LimitExceeded)?;
        let connection = self.connect()?;
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| Failure::WriteFailed)?;
        // The Rust owner supplies digest/bytes. The adapter neither hashes nor repairs them.
        match read_payload(&connection, &payload.digest) {
            Ok(Some(existing)) if existing == *payload => {
                if let Some(row)=custody {
                    if !initial_custody_matches(&connection,row)? {return Err(Failure::ConflictingRetry);}
                }
                return Ok(PreparedWriteV1 {
                    connection: Some(connection),
                    path: self.path.clone(),
                    root_lease: self.root_lease.clone(),
                    #[cfg(windows)] pinned_root: self.pinned_root.clone(),
                })
            }
            Ok(Some(_)) => {
                connection
                    .execute_batch("ROLLBACK")
                    .map_err(|_| Failure::RecoveryRequired)?;
                return Err(Failure::ConflictingRetry);
            }
            Err(error) => {
                connection
                    .execute_batch("ROLLBACK")
                    .map_err(|_| Failure::RecoveryRequired)?;
                return Err(error);
            }
            Ok(None) => {}
        }
        let b = &payload.binding;
        let parent = if b.expected_parent.is_empty() {
            None
        } else {
            Some(b.expected_parent.as_slice())
        };
        let result = connection.execute(
            "INSERT INTO vc04_prepared_payloads VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                payload.digest.as_slice(),
                b.project,
                parent,
                b.new_revision,
                b.execution_profile,
                b.pesi.as_slice(),
                payload.snapshot,
                length,
                b.snapshot_sha256.as_slice(),
                b.revision_integrity.as_slice(),
                payload.envelope
            ],
        );
        let result=result.and_then(|_| {
            if let Some(row)=custody {
                #[cfg(test)]
                if FAIL_INITIAL_CUSTODY_INSERT.with(|fault|fault.replace(false)) {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                connection.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,0,1,?2,?3,?4,?5)",
                    params![row.digest().as_slice(),[0u8;32].as_slice(),row.request(),row.record(),row.hash().as_slice()])?;
            }
            Ok(())
        });
        if result.is_err() {
            if connection.execute_batch("ROLLBACK").is_err() {
                return Err(Failure::RecoveryRequired);
            }
            return Err(Failure::WriteFailed);
        }
        Ok(PreparedWriteV1 {
            connection: Some(connection),
            path: self.path.clone(),
            root_lease: self.root_lease.clone(),
            #[cfg(windows)] pinned_root: self.pinned_root.clone(),
        })
    }

    fn check_handle(&self, handle: &PreparedWriteV1) -> Result<(), Failure> {
        let lease_matches = match (&self.root_lease, &handle.root_lease) {
            (Some(owner), Some(held)) => std::sync::Arc::ptr_eq(owner, held),
            (None, None) => true,
            _ => false,
        };
        if self.path != handle.path || !lease_matches {
            return Err(Failure::InvalidInput);
        }
        #[cfg(windows)]
        {
            match (&self.pinned_root, &handle.pinned_root) {
                (Some(owner), Some(held)) if std::sync::Arc::ptr_eq(owner, held) => {
                    if let Some(connection) = &handle.connection { owner.verify_connection(connection)?; }
                }
                (None, None) => {},
                _ => return Err(Failure::CustodyConflict),
            }
        }
        Ok(())
    }

    pub(crate) fn sync_prepared(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> {
        let _observation = crate::storage_observation::Span::start("sqlite.sync_prepared_full_commit_total");
        self.check_handle(handle)?;
        let connection = handle.connection.take().ok_or(Failure::RecoveryRequired)?;
        connection
            .execute_batch("COMMIT")
            .map_err(|_| Failure::RecoveryRequired)?;
        #[cfg(test)]
        if self.schema_profile==NativeSchemaProfile::RecoveryCustody
            && FAIL_CUSTODY_SYNC_ACK.with(|fault|fault.replace(false)) {return Err(Failure::RecoveryRequired);}
        Ok(())
    }
    pub(crate) fn reread_prepared(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<StoredPayloadV1>, Failure> {
        read_payload(&self.connect()?, digest)
    }

    #[cfg(windows)]
    pub(super) fn reread_initial_prepared_with_custody(&self,request:&super::ActualPreparedRequestV1,
        row:&super::recovery_custody::InitialCustodyRowV1)->Result<StoredPayloadV1,Failure> {
        if self.schema_profile!=NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
        row.check_request(request)?;
        #[cfg(test)]
        if FAIL_CUSTODY_PREPARED_REREAD.with(|fault|fault.replace(false)) {return Err(Failure::RereadFailed);}
        // One new connection/read snapshot covers both halves. No earlier
        // payload read or registry acknowledgement substitutes for this read.
        let connection=self.connect()?;
        connection.execute_batch("BEGIN DEFERRED").map_err(|_|Failure::RereadFailed)?;
        let payload=read_payload(&connection,row.digest())?.ok_or(Failure::RereadFailed)?;
        if &payload!=request.payload()
            || !initial_custody_matches(&connection,row)? {return Err(Failure::RereadMismatch);}
        Ok(payload)
    }

    #[cfg(windows)]
    pub(super) fn append_verified_initial_custody(&self,start:&DurableAttemptStartV1,
        source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        initial:&super::recovery_custody::InitialCustodyRowV1,verified:&super::recovery_custody::VerifiedCustodyRowV1)->Result<(),Failure> {
        start.check_current(self,source)?;verified.check_initial(initial,start.request())?;
        let connection=self.connect()?;
        connection.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::WriteFailed)?;
        if !initial_custody_matches(&connection,initial)? {return Err(Failure::CustodyConflict);}
        #[cfg(test)]
        if FAIL_VERIFIED_CUSTODY_INSERT.with(|fault|fault.replace(false)) {return Err(Failure::WriteFailed);}
        connection.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,1,1,?2,?3,?4,?5)",
            params![initial.digest().as_slice(),initial.hash().as_slice(),initial.request(),verified.record(),verified.hash().as_slice()])
            .map_err(|_|Failure::WriteFailed)?;
        connection.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)?;
        drop(connection);
        // A new read snapshot must recheck both exact rows, not just the new
        // row or the SQL INSERT result, before the caller can attempt CAS.
        let connection=self.connect()?;
        connection.execute_batch("BEGIN DEFERRED").map_err(|_|Failure::RereadFailed)?;
        if !verified_custody_matches(&connection,initial,verified)? {return Err(Failure::RereadMismatch);}
        Ok(())
    }

    #[cfg(windows)]
    pub(super) fn commit_initial_verified_with_custody(&self,start:&DurableAttemptStartV1,
        source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        initial:&super::recovery_custody::InitialCustodyRowV1,verified:&super::recovery_custody::VerifiedCustodyRowV1,
        payload:&StoredPayloadV1,expected:Option<&StoredHeadV1>)->Result<(),Failure> {
        start.check_current(self,source)?;verified.check_initial(initial,start.request())?;
        if expected.is_some() || payload!=start.request().payload() {return Err(Failure::CustodyConflict);}
        self.commit_verified_rows(payload,expected,Some((initial,verified)))
    }

    #[cfg(windows)]
    pub(super) fn retain_initial_outcome(&self,actual:&super::retained_attempt::UnretainedCustodyOutcomeV1,
        row:&super::recovery_custody::ActualOutcomeCustodyRowV1,
        source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>)->Result<(),Failure> {
        actual.start.check_current(self,source)?;row.check_outcome(actual)?;
        let connection=self.connect()?;
        connection.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::WriteFailed)?;
        // An issued Verified row whose acknowledgement was lost must still be
        // present in full. Never reinterpret its absence as a shorter attempt.
        let prefix=if let Some(verified)=&actual.issued_verified {
            verified_custody_matches(&connection,&actual.initial,verified)?
        }else{initial_custody_matches(&connection,&actual.initial)?};
        if !prefix {return Err(Failure::CustodyConflict);}
        #[cfg(test)]
        if FAIL_OUTCOME_CUSTODY.with(|fault|fault.get()==Some(0)) {
            FAIL_OUTCOME_CUSTODY.with(|fault|fault.set(None));return Err(Failure::WriteFailed);
        }
        connection.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,?2,1,?3,?4,?5,?6)",
            params![actual.initial.digest().as_slice(),row.ordinal(),row.previous().as_slice(),actual.initial.request(),row.record(),row.hash().as_slice()])
            .map_err(|_|Failure::WriteFailed)?;
        connection.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)?;
        drop(connection);
        #[cfg(test)]
        if FAIL_OUTCOME_CUSTODY.with(|fault|fault.get()==Some(1)) {
            FAIL_OUTCOME_CUSTODY.with(|fault|fault.set(None));return Err(Failure::RereadFailed);
        }
        self.reread_initial_outcome(actual,row)
    }

    /// New connection and one read snapshot for payload and the entire exact
    /// prefix. This confirms retained bytes, not root/epoch or a receipt.
    #[cfg(windows)]
    pub(super) fn reread_initial_outcome(&self,actual:&super::retained_attempt::UnretainedCustodyOutcomeV1,
        row:&super::recovery_custody::ActualOutcomeCustodyRowV1)->Result<(),Failure> {
        if self.schema_profile!=NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
        row.check_outcome(actual)?;
        let connection=self.connect()?;
        connection.execute_batch("BEGIN DEFERRED").map_err(|_|Failure::RereadFailed)?;
        let initial=&actual.initial;
        if read_payload(&connection,initial.digest())?.as_ref()!=Some(actual.start.request().payload())
            || !custody_count_matches(&connection,initial.digest(),i64::from(row.ordinal())+1)?
            || !custody_row_matches(&connection,initial.digest(),0,&[0;32],initial.request(),initial.record(),initial.hash())? {
            return Err(Failure::RereadMismatch);
        }
        if let Some(verified)=&actual.issued_verified {
            if !custody_row_matches(&connection,initial.digest(),1,initial.hash(),initial.request(),verified.record(),verified.hash())? {
                return Err(Failure::RereadMismatch);
            }
        }
        if !custody_row_matches(&connection,initial.digest(),row.ordinal(),row.previous(),initial.request(),row.record(),row.hash())? {
            return Err(Failure::RereadMismatch);
        }
        Ok(())
    }

    /// Mechanical cold supplier: metadata before borrowed BLOBs, one row at a
    /// time in the same read snapshot. The common visitor owns state/hash
    /// validation. This method cannot grant root/Attempt/restore authority.
    #[cfg(windows)]
    pub(super) fn visit_initial_custody_rows<F>(&self,expected:&StoredPayloadV1,mut visit:F)->Result<(),Failure>
    where F:FnMut(u8,&[u8],&[u8],&[u8],&[u8])->Result<(),Failure> {
        if self.schema_profile!=NativeSchemaProfile::RecoveryCustody {return Err(Failure::UnsupportedDurability);}
        let connection=self.connect()?;
        connection.execute_batch("BEGIN DEFERRED").map_err(|_|Failure::RereadFailed)?;
        self.visit_custody_on_connection(&connection,expected,&mut visit)
    }

    #[cfg(windows)]
    fn visit_custody_on_connection<F>(&self,connection:&Connection,expected:&StoredPayloadV1,mut visit:F)->Result<(),Failure>
    where F:FnMut(u8,&[u8],&[u8],&[u8],&[u8])->Result<(),Failure> {
        if !initial_payload_matches(connection,expected)? {return Err(Failure::RereadMismatch);}
        let mut statement=connection.prepare("SELECT journal_ordinal,custody_version,typeof(previous_row_sha256),length(previous_row_sha256),typeof(admitted_request_bytes),length(admitted_request_bytes),typeof(recovery_record_bytes),length(recovery_record_bytes),typeof(row_sha256),length(row_sha256) FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 ORDER BY journal_ordinal")
            .map_err(|_|Failure::RereadFailed)?;
        let mut rows=statement.query([expected.digest.as_slice()]).map_err(|_|Failure::RereadFailed)?;
        let mut count=0usize;
        while let Some(meta)=rows.next().map_err(|_|Failure::RereadFailed)? {
            count+=1;
            let ordinal=meta.get::<_,i64>(0).map_err(|_|Failure::CorruptRecoveryRecord)?;
            if count>64 || ordinal>=64 {return Err(Failure::LimitExceeded);}
            if ordinal<0 {return Err(Failure::CorruptRecoveryRecord);}
            let version=meta.get::<_,i64>(1).map_err(|_|Failure::CorruptRecoveryRecord)?;
            let mut lengths=[0usize;4];
            for (i,(type_column,length_column,max)) in [(2,3,32usize),(4,5,super::recovery_custody::REQUEST_MAX),
                (6,7,super::recovery_custody::RECORD_MAX),(8,9,32usize)].into_iter().enumerate() {
                let kind=meta.get_ref(type_column).map_err(|_|Failure::CorruptRecoveryRecord)?;
                let length=meta.get::<_,i64>(length_column).map_err(|_|Failure::CorruptRecoveryRecord)?;
                if length<1 || length as u64>max as u64 {return Err(Failure::LimitExceeded);}
                if kind!=rusqlite::types::ValueRef::Text(b"blob") {return Err(Failure::CorruptRecoveryRecord);}
                lengths[i]=length as usize;
            }
            if version!=1 || lengths[0]!=32 || lengths[3]!=32 {return Err(Failure::CorruptRecoveryRecord);}
            let checked=connection.query_row("SELECT previous_row_sha256,admitted_request_bytes,recovery_record_bytes,row_sha256 FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=?2",
                params![expected.digest.as_slice(),ordinal],|row| {
                    let previous=row.get_ref(0)?.as_blob()?;let request=row.get_ref(1)?.as_blob()?;
                    let record=row.get_ref(2)?.as_blob()?;let hash=row.get_ref(3)?.as_blob()?;
                    if [previous.len(),request.len(),record.len(),hash.len()]!=lengths {return Ok(Err(Failure::RereadMismatch));}
                    Ok(visit(ordinal as u8,previous,request,record,hash))
                }).map_err(|_|Failure::RereadFailed)?;
            checked?;
        }
        if count==0 {return Err(Failure::RecoveryRequired);}
        Ok(())
    }

    #[cfg(windows)]
    pub(super) fn retain_recovery_outcome(&self,actual:&super::retained_attempt::UnretainedRecoveryOutcomeV1,
        row:&super::recovery_custody::RecoveryOutcomeCustodyRowV1,
        source:&crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    )->Result<(),Failure> {
        use super::recovery_custody::InitialPrefixVerifierV1;
        actual.start.check_current(self,source)?;row.check_outcome(actual)?;
        let connection=self.connect()?;
        connection.execute_batch("BEGIN IMMEDIATE").map_err(|_|Failure::WriteFailed)?;
        let mut verifier=InitialPrefixVerifierV1::new(actual.start.expected_request(),actual.start.prior_hash(),actual.start.prior_epoch());
        self.visit_custody_on_connection(&connection,actual.start.expected_payload(),|ordinal,previous,request,record,hash| {
            verifier.accept(ordinal,previous,request,record,hash)
        })?;
        if verifier.finish()?.record()!=actual.start.prior_record().record() {return Err(Failure::CustodyConflict);}
        #[cfg(test)]
        if FAIL_OUTCOME_CUSTODY.with(|fault|fault.get()==Some(0)) {
            FAIL_OUTCOME_CUSTODY.with(|fault|fault.set(None));return Err(Failure::WriteFailed);
        }
        connection.execute("INSERT INTO vc04_recovery_custody_v1 VALUES (?1,?2,1,?3,?4,?5,?6)",
            params![actual.start.expected_payload().digest.as_slice(),row.ordinal(),row.previous().as_slice(),actual.start.expected_raw(),row.record(),row.hash().as_slice()])
            .map_err(|_|Failure::WriteFailed)?;
        connection.execute_batch("COMMIT").map_err(|_|Failure::RecoveryRequired)?;drop(connection);
        #[cfg(test)]
        if FAIL_OUTCOME_CUSTODY.with(|fault|fault.get()==Some(1)) {
            FAIL_OUTCOME_CUSTODY.with(|fault|fault.set(None));return Err(Failure::RereadFailed);
        }
        self.reread_recovery_outcome(actual,row)
    }

    #[cfg(windows)]
    pub(super) fn reread_recovery_outcome(&self,actual:&super::retained_attempt::UnretainedRecoveryOutcomeV1,
        row:&super::recovery_custody::RecoveryOutcomeCustodyRowV1,
    )->Result<(),Failure> {
        use sha2::{Digest,Sha256};
        use super::recovery_custody::InitialPrefixVerifierV1;
        row.check_outcome(actual)?;
        let mut verifier=InitialPrefixVerifierV1::new(actual.start.expected_request(),Sha256::digest(row.record()).into(),actual.start.epoch());
        self.visit_initial_custody_rows(actual.start.expected_payload(),|ordinal,previous,request,record,hash| {
            verifier.accept(ordinal,previous,request,record,hash)
        })?;
        let latest=verifier.finish()?;
        if latest.record()!=row.record() || latest.ordinal()!=row.ordinal() || latest.row_hash()!=row.hash() {
            return Err(Failure::RereadMismatch);
        }
        Ok(())
    }

    /// Expected head includes the full existing RD-631 triple plus correlation digest.
    /// Result is SQL mechanics only; success cannot issue Committed or a receipt.
    pub(crate) fn commit_verified(
        &self,
        payload: &StoredPayloadV1,
        expected: Option<&StoredHeadV1>,
    ) -> Result<(), Failure> {
        if self.schema_profile!=NativeSchemaProfile::Base {return Err(Failure::UnsupportedDurability);}
        self.commit_verified_rows(payload,expected,None)
    }

    fn commit_verified_rows(&self,payload:&StoredPayloadV1,expected:Option<&StoredHeadV1>,
        custody:Option<(&super::recovery_custody::InitialCustodyRowV1,&super::recovery_custody::VerifiedCustodyRowV1)>)->Result<(),Failure> {
        let _observation = crate::storage_observation::Span::start("sqlite.commit_verified_total");
        #[cfg(all(windows,feature="beta2-crash-probe"))]
        crash_probe::checkpoint("before_cas");
        #[cfg(windows)]
        { self.fresh_container.try_borrow_mut().map_err(|_| Failure::CustodyConflict)?.take(); }
        let connection = self.connect()?;
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| Failure::AtomicCommitFailed)?;
        let result = (|| {
            if let Some((initial,verified))=custody {
                if !verified_custody_matches(&connection,initial,verified)? {return Err(Failure::CustodyConflict);}
            }
            if read_payload(&connection, &payload.digest)?.as_ref() != Some(payload) {
                return Err(Failure::RereadMismatch);
            }
            let b = &payload.binding;
            if expected
                .map(|head| {
                    head.project.as_slice() != b.project || head.revision != b.expected_parent
                })
                .unwrap_or(!b.expected_parent.is_empty())
            {
                return Err(Failure::CustodyConflict);
            }
            if read_head(&connection, "vc04_project_heads", &b.project, None)?.as_ref() != expected
            {
                return Err(Failure::StaleRevision);
            }
            connection
                .execute(
                    "INSERT INTO vc04_project_revisions VALUES (?1,?2,?3,?4)",
                    params![
                        b.project,
                        b.new_revision,
                        b.revision_integrity.as_slice(),
                        payload.digest.as_slice()
                    ],
                )
                .map_err(|_| Failure::ConflictingRetry)?;
            let changed=if let Some(old)=expected {
                connection.execute("UPDATE vc04_project_heads SET history_revision_identity=?1,revision_integrity_sha256=?2,transaction_digest=?3
                    WHERE project_identity=?4 AND history_revision_identity=?5 AND revision_integrity_sha256=?6 AND transaction_digest=?7",
                    params![b.new_revision,b.revision_integrity.as_slice(),payload.digest.as_slice(),b.project,old.revision,old.integrity.as_slice(),old.digest.as_slice()])
            } else {
                connection.execute("INSERT INTO vc04_project_heads VALUES (?1,?2,?3,?4)",params![b.project,b.new_revision,b.revision_integrity.as_slice(),payload.digest.as_slice()])
            }.map_err(|_|Failure::AtomicCommitFailed)?;
            if changed != 1 {
                return Err(Failure::StaleRevision);
            }
            Ok(())
        })();
        if let Err(error) = result {
            if connection.execute_batch("ROLLBACK").is_err() {
                return Err(Failure::RecoveryRequired);
            }
            return Err(error);
        }
        connection
            .execute_batch("COMMIT")
            .map_err(|_| Failure::RecoveryRequired)?;
        #[cfg(all(windows,feature="beta2-crash-probe"))]
        crash_probe::checkpoint("after_cas");
        Ok(())
    }

    pub(crate) fn confirm_container_durable(&self) -> Result<(), Failure> {
        let _observation = crate::storage_observation::Span::start("sqlite.confirm_container_durable_total");
        #[cfg(test)]
        if self.schema_profile==NativeSchemaProfile::RecoveryCustody
            && FAIL_CUSTODY_CONFIRM.with(|fault|fault.replace(false)) {return Err(Failure::ContainerDurabilityFailed);}
        self.capability()?;
        let connection = self.connect()?;
        let checkpoint_observation = crate::storage_observation::Span::start("sqlite.wal_checkpoint_full");
        let (busy, frames, checkpointed): (i64, i64, i64) = connection
            .query_row("PRAGMA wal_checkpoint(FULL)", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|_| Failure::ContainerDurabilityFailed)?;
        drop(checkpoint_observation);
        if busy != 0 || frames < 0 || frames != checkpointed {
            return Err(Failure::ContainerDurabilityFailed);
        }
        // Confirm the post-checkpoint container, not only its pre-checkpoint
        // capability. Persisted WAL/SHM must remain in the admitted root.
        self.capability()?;
        Ok(())
    }
    pub(crate) fn reread_committed(
        &self,
        digest: &[u8; 32],
        project: &[u8],
        revision: &[u8],
    ) -> Result<RawChainV1, Failure> {
        #[cfg(test)]
        if self.schema_profile==NativeSchemaProfile::RecoveryCustody
            && FAIL_CUSTODY_COMMITTED_REREAD.with(|fault|fault.replace(false)) {return Err(Failure::RereadFailed);}
        let connection = self.connect()?;
        connection.execute_batch("BEGIN").map_err(database_error)?;
        let head = read_head(&connection, "vc04_project_heads", project, None)?;
        let chain = RawChainV1 {
            payload: read_payload(&connection, digest)?,
            revision: read_head(
                &connection,
                "vc04_project_revisions",
                project,
                Some(revision),
            )?,
            head,
        };
        connection.execute_batch("COMMIT").map_err(database_error)?;
        Ok(chain)
    }
    pub(crate) fn abort(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> {
        self.check_handle(handle)?;
        if let Some(connection) = handle.connection.take() {
            connection
                .execute_batch("ROLLBACK")
                .map_err(|_| Failure::RecoveryRequired)?;
        }
        // Already durable orphan rows remain untouched and nonauthoritative.
        Ok(())
    }
    pub(crate) fn inspect_recovery(
        &self,
        digest: &[u8; 32],
        project: &[u8],
        revision: &[u8],
    ) -> Result<RawChainV1, Failure> {
        self.reread_committed(digest, project, revision)
    }
}

impl super::native_adapter_sealed::Sealed for SqliteAdapterV1 {}
impl super::SnapshotV2DurabilityAdapterV1 for SqliteAdapterV1 {
    type PreparedHandle = PreparedWriteV1;
    fn capability(&self) -> Result<&'static str, Failure> { SqliteAdapterV1::capability(self) }
    fn write_prepared(&self, payload: &StoredPayloadV1) -> Result<PreparedWriteV1, Failure> { SqliteAdapterV1::write_prepared(self, payload) }
    fn sync_prepared(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> { SqliteAdapterV1::sync_prepared(self, handle) }
    fn reread_prepared(&self, digest: &[u8;32]) -> Result<Option<StoredPayloadV1>, Failure> { SqliteAdapterV1::reread_prepared(self, digest) }
    fn commit_verified(&self, payload: &StoredPayloadV1, expected: Option<&StoredHeadV1>) -> Result<(), Failure> { SqliteAdapterV1::commit_verified(self, payload, expected) }
    fn confirm_container_durable(&self) -> Result<(), Failure> { SqliteAdapterV1::confirm_container_durable(self) }
    fn reread_committed(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<RawChainV1, Failure> { SqliteAdapterV1::reread_committed(self, digest, project, revision) }
    fn abort(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> { SqliteAdapterV1::abort(self, handle) }
    fn inspect_recovery(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<RawChainV1, Failure> { SqliteAdapterV1::inspect_recovery(self, digest, project, revision) }
}

fn retain_wal(connection: &Connection) -> Result<(), Failure> {
    let mut persist = 1i32;
    // SAFETY: the connection and stack integer outlive the synchronous SQLite
    // file-control call; bundled SQLite owns the default main VFS.
    let result = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            b"main\0".as_ptr().cast(),
            rusqlite::ffi::SQLITE_FCNTL_PERSIST_WAL,
            (&mut persist as *mut i32).cast(),
        )
    };
    if result != rusqlite::ffi::SQLITE_OK || persist != 1 {
        return Err(Failure::UnsupportedDurability);
    }
    Ok(())
}

fn bound_sqlite_records(connection: &Connection) -> Result<(), Failure> {
    // SAFETY: this live connection exclusively owns the synchronous SQLite
    // handle. A negative new value is the documented non-mutating limit query.
    let actual = unsafe {
        rusqlite::ffi::sqlite3_limit(
            connection.handle(),
            rusqlite::ffi::SQLITE_LIMIT_LENGTH,
            MAX_SQLITE_RECORD_BYTES,
        );
        rusqlite::ffi::sqlite3_limit(connection.handle(), rusqlite::ffi::SQLITE_LIMIT_LENGTH, -1)
    };
    if actual != MAX_SQLITE_RECORD_BYTES {
        return Err(Failure::UnsupportedDurability);
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), Failure> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.write(true).custom_flags(0x02000000);
    }
    options
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| Failure::UnsupportedDurability)
}

fn reject_reparse(path: &Path) -> Result<(), Failure> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| Failure::UnsupportedDurability)?;
    if metadata.file_type().is_symlink() {
        return Err(Failure::UnsupportedDurability);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(Failure::UnsupportedDurability);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn acquire_local_root(root: &Path) -> Result<(PathBuf, std::fs::File), Failure> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::{Component, Prefix};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDriveTypeW(root: *const u16) -> u32;
    }
    if !root.is_absolute() {
        return Err(Failure::UnsupportedDurability);
    }
    for ancestor in root.ancestors() {
        if !ancestor.as_os_str().is_empty() {
            reject_reparse(ancestor)?;
        }
    }
    let root = root
        .canonicalize()
        .map_err(|_| Failure::UnsupportedDurability)?;
    let drive = match root.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => letter,
            _ => return Err(Failure::UnsupportedDurability),
        },
        _ => return Err(Failure::UnsupportedDurability),
    };
    let drive_root = [drive as u16, b':' as u16, b'\\' as u16, 0];
    // SAFETY: zero-terminated UTF-16 root remains live for the synchronous call.
    if unsafe { GetDriveTypeW(drive_root.as_ptr()) } != 3 {
        return Err(Failure::UnsupportedDurability);
    }
    if !root.is_dir() {
        return Err(Failure::UnsupportedDurability);
    }
    let lock = root.join("vc04_adapter.lock");
    if lock.exists() {
        reject_reparse(&lock)?;
    }
    let lease = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .share_mode(0)
        .custom_flags(0x00200000)
        .open(&lock)
        .map_err(|_| Failure::UnsupportedDurability)?;
    use std::os::windows::fs::MetadataExt;
    let metadata = lease
        .metadata()
        .map_err(|_| Failure::UnsupportedDurability)?;
    if metadata.len() != 0 || metadata.file_attributes() & 0x400 != 0 || !metadata.is_file() {
        return Err(Failure::UnsupportedDurability);
    }
    // The zero-share live handle coordinates every admitted writer/maintenance
    // connection. Closing it releases the lease; no lock file is deleted.
    Ok((root, lease))
}

#[cfg(not(windows))]
fn acquire_local_root(_: &Path) -> Result<(PathBuf, std::fs::File), Failure> {
    // No unverified platform-specific local-volume/exclusive-root substitute.
    Err(Failure::UnsupportedDurability)
}

#[cfg(test)]
std::thread_local! {static FAIL_INITIAL_CUSTODY_INSERT:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
#[cfg(test)]
std::thread_local! {static FAIL_VERIFIED_CUSTODY_INSERT:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
#[cfg(test)]
std::thread_local! {static FAIL_OUTCOME_CUSTODY:std::cell::Cell<Option<u8>>=const {std::cell::Cell::new(None)};}
#[cfg(test)]
std::thread_local! {static FAIL_CUSTODY_PREPARED_REREAD:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
#[cfg(test)]
std::thread_local! {static FAIL_CUSTODY_CONFIRM:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
#[cfg(test)]
std::thread_local! {static FAIL_CUSTODY_COMMITTED_REREAD:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
#[cfg(test)]
std::thread_local! {static FAIL_CUSTODY_SYNC_ACK:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}

fn initial_custody_matches(connection:&Connection,expected:&super::recovery_custody::InitialCustodyRowV1)->Result<bool,Failure> {
    Ok(custody_count_matches(connection,expected.digest(),1)?
        && custody_row_matches(connection,expected.digest(),0,&[0;32],expected.request(),expected.record(),expected.hash())?)
}

fn verified_custody_matches(connection:&Connection,initial:&super::recovery_custody::InitialCustodyRowV1,
    verified:&super::recovery_custody::VerifiedCustodyRowV1)->Result<bool,Failure> {
    Ok(custody_count_matches(connection,initial.digest(),2)?
        && custody_row_matches(connection,initial.digest(),0,&[0;32],initial.request(),initial.record(),initial.hash())?
        && custody_row_matches(connection,initial.digest(),1,initial.hash(),initial.request(),verified.record(),verified.hash())?)
}

fn custody_count_matches(connection:&Connection,digest:&[u8;32],expected:i64)->Result<bool,Failure> {
    let count:i64=connection.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 LIMIT 65)",
        [digest.as_slice()],|row|row.get(0)).map_err(|_|Failure::RereadFailed)?;
    if count>64 {return Err(Failure::LimitExceeded);}
    Ok(count==expected)
}

#[cfg(windows)]
fn initial_payload_matches(connection:&Connection,expected:&StoredPayloadV1)->Result<bool,Failure> {
    use rusqlite::types::ValueRef::{Blob,Null};
    if !expected.binding.expected_parent.is_empty() {return Err(Failure::UnsupportedDurability);}
    let b=&expected.binding;
    connection.query_row("SELECT * FROM vc04_prepared_payloads WHERE transaction_digest=?1",[expected.digest.as_slice()],|row| {
        Ok(matches!(row.get_ref(0)?,Blob(value) if value==expected.digest)
            && matches!(row.get_ref(1)?,Blob(value) if value==b.project)
            && matches!(row.get_ref(2)?,Null)
            && matches!(row.get_ref(3)?,Blob(value) if value==b.new_revision)
            && matches!(row.get_ref(4)?,Blob(value) if value==b.execution_profile)
            && matches!(row.get_ref(5)?,Blob(value) if value==b.pesi)
            && matches!(row.get_ref(6)?,Blob(value) if value==expected.snapshot)
            && row.get::<_,i64>(7)?==i64::try_from(b.snapshot_length).unwrap_or(-1)
            && matches!(row.get_ref(8)?,Blob(value) if value==b.snapshot_sha256)
            && matches!(row.get_ref(9)?,Blob(value) if value==b.revision_integrity)
            && matches!(row.get_ref(10)?,Blob(value) if value==expected.envelope))
    }).optional().map(|value|value.unwrap_or(false)).map_err(|_|Failure::RereadFailed)
}

fn custody_row_matches(connection:&Connection,digest:&[u8;32],ordinal:u8,previous:&[u8;32],request:&[u8],record:&[u8],hash:&[u8;32])->Result<bool,Failure> {
    // Only small metadata crosses into owned Rust values before the BLOB gate.
    let metadata=connection.query_row("SELECT custody_version,typeof(previous_row_sha256),length(previous_row_sha256),typeof(admitted_request_bytes),length(admitted_request_bytes),typeof(recovery_record_bytes),length(recovery_record_bytes),typeof(row_sha256),length(row_sha256) FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=?2",
        params![digest.as_slice(),ordinal],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?,
            row.get::<_,String>(3)?,row.get::<_,i64>(4)?,row.get::<_,String>(5)?,row.get::<_,i64>(6)?,row.get::<_,String>(7)?,row.get::<_,i64>(8)?)))
        .optional().map_err(|_|Failure::RereadFailed)?;
    let Some((version,pt,pl,qt,ql,rt,rl,ht,hl))=metadata else{return Ok(false);};
    if ql<1 || ql as u64>super::recovery_custody::REQUEST_MAX as u64 || rl<1 || rl as u64>super::recovery_custody::RECORD_MAX as u64 {
        return Err(Failure::LimitExceeded);
    }
    if version!=1 || pt!="blob" || pl!=32 || qt!="blob" || rt!="blob" || ht!="blob" || hl!=32
        || ql as usize!=request.len() || rl as usize!=record.len() {return Ok(false);}
    connection.query_row("SELECT previous_row_sha256,admitted_request_bytes,recovery_record_bytes,row_sha256 FROM vc04_recovery_custody_v1 WHERE transaction_digest=?1 AND journal_ordinal=?2",
        params![digest.as_slice(),ordinal],|row|{
            use rusqlite::types::ValueRef::Blob;
            Ok(matches!(row.get_ref(0)?,Blob(value) if value==previous)
                && matches!(row.get_ref(1)?,Blob(value) if value==request)
                && matches!(row.get_ref(2)?,Blob(value) if value==record)
                && matches!(row.get_ref(3)?,Blob(value) if value==hash))
        }).map_err(|_|Failure::RereadFailed)
}

fn read_payload(
    connection: &Connection,
    digest: &[u8; 32],
) -> Result<Option<StoredPayloadV1>, Failure> {
    connection
        .query_row(
            "SELECT * FROM vc04_prepared_payloads WHERE transaction_digest=?1",
            [digest.as_slice()],
            |row| {
                let length: i64 = row.get(7)?;
                // Schema checks protect writes by this adapter, not hostile or
                // damaged persisted rows. Inspect borrowed SQLite values before
                // Vec allocation; never trust a saved length or cast negatives.
                for index in [1, 3, 4] {
                    let value = row.get_ref(index)?.as_blob()?;
                    if value.is_empty() || value.len() > 4096 {
                        return Err(rusqlite::Error::InvalidQuery);
                    }
                }
                match row.get_ref(2)? {
                    rusqlite::types::ValueRef::Null => {}
                    rusqlite::types::ValueRef::Blob(value)
                        if !value.is_empty() && value.len() <= 4096 => {}
                    _ => return Err(rusqlite::Error::InvalidQuery),
                }
                for index in [0, 5, 8, 9] {
                    if row.get_ref(index)?.as_blob()?.len() != 32 {
                        return Err(rusqlite::Error::InvalidQuery);
                    }
                }
                let snapshot = row.get_ref(6)?.as_blob()?;
                let envelope = row.get_ref(10)?.as_blob()?;
                if length <= 0
                    || length > 2_000_000
                    || snapshot.len() as u64 != length as u64
                    || envelope.is_empty()
                    || envelope.len() > 1_000_000
                {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                Ok(StoredPayloadV1 {
                    digest: row.get(0)?,
                    binding: TransactionBindingV1 {
                        project: row.get(1)?,
                        expected_parent: row.get::<_, Option<Vec<u8>>>(2)?.unwrap_or_default(),
                        new_revision: row.get(3)?,
                        execution_profile: row.get(4)?,
                        pesi: row.get(5)?,
                        snapshot_length: length as u64,
                        snapshot_sha256: row.get(8)?,
                        revision_integrity: row.get(9)?,
                    },
                    snapshot: row.get(6)?,
                    envelope: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(database_error)
}
fn read_head(
    connection: &Connection,
    table: &str,
    project: &[u8],
    revision: Option<&[u8]>,
) -> Result<Option<StoredHeadV1>, Failure> {
    let select=format!("SELECT project_identity,history_revision_identity,revision_integrity_sha256,transaction_digest FROM {table} WHERE project_identity=?1");
    let row = |row: &rusqlite::Row<'_>| {
        for index in [0, 1] {
            let value = row.get_ref(index)?.as_blob()?;
            if value.is_empty() || value.len() > 4096 {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }
        for index in [2, 3] {
            if row.get_ref(index)?.as_blob()?.len() != 32 {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }
        Ok(StoredHeadV1 {
            project: row.get(0)?,
            revision: row.get(1)?,
            integrity: row.get(2)?,
            digest: row.get(3)?,
        })
    };
    match revision {
        Some(revision) => connection.query_row(
            &format!("{select} AND history_revision_identity=?2"),
            params![project, revision],
            row,
        ),
        None => connection.query_row(&select, [project], row),
    }
    .optional()
    .map_err(database_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FaultPoint { None, PreparedWrite, BeforePreparedCommit, PreparedReread, ContainerConfirmation, CommittedReread }

    #[cfg(windows)]
    struct FaultAdapter<'a> {
        inner: &'a SqliteAdapterV1,
        fault: FaultPoint,
        calls: std::cell::RefCell<Vec<&'static str>>,
    }

    #[cfg(windows)]
    impl crate::snapshot_v2_durable_transaction::native_adapter_sealed::Sealed for FaultAdapter<'_> {}
    #[cfg(windows)]
    impl crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityAdapterV1 for FaultAdapter<'_> {
        type PreparedHandle = PreparedWriteV1;
        fn capability(&self) -> Result<&'static str, Failure> { self.calls.borrow_mut().push("capability"); self.inner.capability() }
        fn write_prepared(&self, payload: &StoredPayloadV1) -> Result<PreparedWriteV1, Failure> {
            self.calls.borrow_mut().push("write_prepared");
            if self.fault == FaultPoint::PreparedWrite { return Err(Failure::WriteFailed); }
            self.inner.write_prepared(payload)
        }
        fn sync_prepared(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> {
            self.calls.borrow_mut().push("sync_prepared");
            if self.fault == FaultPoint::BeforePreparedCommit { return Err(Failure::SyncFailed); }
            self.inner.sync_prepared(handle)
        }
        fn reread_prepared(&self, digest: &[u8;32]) -> Result<Option<StoredPayloadV1>, Failure> {
            self.calls.borrow_mut().push("reread_prepared");
            let mut row = self.inner.reread_prepared(digest)?;
            if self.fault == FaultPoint::PreparedReread { row.as_mut().unwrap().snapshot[0] ^= 1; }
            Ok(row)
        }
        fn commit_verified(&self, payload: &StoredPayloadV1, expected: Option<&StoredHeadV1>) -> Result<(), Failure> {
            self.calls.borrow_mut().push("commit_verified"); self.inner.commit_verified(payload, expected)
        }
        fn confirm_container_durable(&self) -> Result<(), Failure> {
            self.calls.borrow_mut().push("confirm_container_durable");
            if self.fault == FaultPoint::ContainerConfirmation { return Err(Failure::ContainerDurabilityFailed); }
            self.inner.confirm_container_durable()
        }
        fn reread_committed(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<RawChainV1, Failure> {
            self.calls.borrow_mut().push("reread_committed");
            if self.fault == FaultPoint::CommittedReread { return Err(Failure::RereadFailed); }
            self.inner.reread_committed(digest, project, revision)
        }
        fn abort(&self, handle: &mut PreparedWriteV1) -> Result<(), Failure> {
            self.calls.borrow_mut().push("abort"); self.inner.abort(handle)
        }
        fn inspect_recovery(&self, digest: &[u8;32], project: &[u8], revision: &[u8]) -> Result<RawChainV1, Failure> {
            self.calls.borrow_mut().push("inspect_recovery"); self.inner.inspect_recovery(digest, project, revision)
        }
    }

    #[cfg(windows)]
    #[test]
    fn shared_native_execution_orders_real_sqlite_and_preserves_failure_disposition() {
        use crate::snapshot_v2_durable_transaction::{commit_native_evidence, SnapshotV2TransactionStateV1 as State};
        use sha2::{Digest, Sha256};
        let checked = |revision, parent| {
            let mut row = payload(revision, parent);
            row.binding.snapshot_sha256 = Sha256::digest(&row.snapshot).into();
            row.digest = row.binding.transaction_digest().unwrap(); row
        };
        for fault in [FaultPoint::None, FaultPoint::BeforePreparedCommit, FaultPoint::PreparedReread,
            FaultPoint::ContainerConfirmation, FaultPoint::CommittedReread] {
            let store = SqliteAdapterV1::provision(&empty_root()).unwrap();
            let first = checked(1, vec![]);
            let first_evidence = commit_native_evidence(&store, &first, None, b"vc04.sqlite-durability.r1").unwrap();
            assert_eq!(first_evidence.state(), State::Committed);
            let old = store.reread_committed(&first.digest, b"project", &[1]).unwrap();
            let second = checked(2, vec![1]);
            let adapter = FaultAdapter { inner: &store, fault, calls: Default::default() };
            let result = commit_native_evidence(&adapter, &second, old.head.as_ref(), b"vc04.sqlite-durability.r1");
            match fault {
                FaultPoint::None => {
                    assert_eq!(result.unwrap().state(), State::Committed);
                    assert_eq!(*adapter.calls.borrow(), ["capability", "write_prepared", "sync_prepared", "reread_prepared",
                        "commit_verified", "confirm_container_durable", "reread_committed"]);
                }
                _ => {
                    let failure = result.unwrap_err();
                    let after_cas = matches!(fault, FaultPoint::ContainerConfirmation | FaultPoint::CommittedReread);
                    assert_eq!(failure.state, Some(if after_cas { State::RecoveryRequired } else { State::Aborted }));
                    assert_eq!(failure.cause, match fault { FaultPoint::BeforePreparedCommit => Failure::SyncFailed,
                        FaultPoint::PreparedReread => Failure::RereadMismatch,
                        FaultPoint::ContainerConfirmation => Failure::ContainerDurabilityFailed,
                        FaultPoint::CommittedReread => Failure::RereadFailed, _ => unreachable!() });
                    let observed = store.inspect_recovery(&first.digest, b"project", &[1]).unwrap();
                    assert_eq!(observed.payload, old.payload); assert_eq!(observed.revision, old.revision);
                    if !after_cas { assert_eq!(observed.head, old.head); }
                    else { assert_eq!(observed.head.unwrap().revision, vec![2]); }
                }
            }
        }
        // These are real SQLite mechanics/evidence fixtures, not admitted
        // source/custody, issued receipts, restored worlds or process-kill proof.
    }

    #[cfg(windows)]
    #[test]
    fn m2_native_owner_outcome_retains_actual_failure_and_matches_result_facade() {
        use crate::snapshot_v2_durable_transaction::{commit_native_evidence,run_native_owner_outcome,SnapshotV2TransactionStateV1 as State};
        use crate::snapshot_v2_durable_transaction::recovery_custody::{RecoveryRecordCandidateV1,
            encode_prepared_candidate,encode_actual_outcome_candidate,check_actual_outcome_candidate};
        use sha2::{Digest,Sha256};
        let mut row=payload(1,vec![]);
        row.binding.snapshot_sha256=Sha256::digest(&row.snapshot).into();
        row.digest=row.binding.transaction_digest().unwrap();
        let initial=encode_prepared_candidate(&row.binding,None).unwrap();
        let mut different_causes=Vec::new();
        for fault in [FaultPoint::PreparedWrite,FaultPoint::BeforePreparedCommit,FaultPoint::ContainerConfirmation,FaultPoint::CommittedReread,FaultPoint::None] {
            let first=SqliteAdapterV1::provision(&empty_root()).unwrap();
            let second=SqliteAdapterV1::provision(&empty_root()).unwrap();
            let retained_adapter=FaultAdapter {inner:&first,fault,calls:Default::default()};
            let legacy_adapter=FaultAdapter {inner:&second,fault,calls:Default::default()};
            let retained=run_native_owner_outcome(&retained_adapter,&row,None,b"vc04.sqlite-durability.r1").unwrap();
            let legacy=commit_native_evidence(&legacy_adapter,&row,None,b"vc04.sqlite-durability.r1");
            assert_eq!(*retained_adapter.calls.borrow(),*legacy_adapter.calls.borrow(),"same loop, not a second state machine");
            assert_eq!(retained.evidence().binding,row.binding);
            assert_eq!(retained.failure(),legacy.as_ref().err(),"the old Result facade must preserve its exact failure");
            if let Some(failure)=retained.failure() {
                assert_eq!(failure.state,Some(retained.evidence().state()));
                assert_eq!(failure.cause,match fault {
                    FaultPoint::PreparedWrite=>Failure::WriteFailed,
                    FaultPoint::BeforePreparedCommit=>Failure::SyncFailed,
                    FaultPoint::ContainerConfirmation=>Failure::ContainerDurabilityFailed,
                    FaultPoint::CommittedReread=>Failure::RereadFailed,
                    _=>unreachable!(),
                });
                assert_eq!(retained.evidence().state(),if matches!(fault,FaultPoint::PreparedWrite|FaultPoint::BeforePreparedCommit){State::Aborted}else{State::RecoveryRequired});
            } else {
                assert_eq!(retained.evidence().state(),State::Committed);
                assert_eq!(legacy.unwrap().state(),State::Committed);
            }
            let encoded=encode_actual_outcome_candidate(&retained,&initial).unwrap();
            let decoded=RecoveryRecordCandidateV1::decode(&encoded).unwrap();
            assert_eq!(decoded.state(),retained.evidence().state());
            assert_eq!(decoded.cause(),retained.failure().map(|failure|failure.cause).unwrap_or(Failure::Interrupted));
            assert_eq!(decoded.confirmed(),retained.evidence().container_confirmed);
            check_actual_outcome_candidate(&encoded,&retained,&initial).unwrap();
            if matches!(fault,FaultPoint::PreparedWrite|FaultPoint::BeforePreparedCommit) {
                different_causes.push((retained,encoded));
            }
        }
        assert_eq!(different_causes.len(),2);
        assert!(check_actual_outcome_candidate(&different_causes[0].1,&different_causes[1].0,&initial).is_err(),"same binding/state does not permit swapping WriteFailed for SyncFailed");
        assert!(check_actual_outcome_candidate(&different_causes[1].1,&different_causes[0].0,&initial).is_err());
        // Actual SQLite/common-owner outcomes only: source/root admission,
        // record Witness persistence, receipt and cold recovery are not proved.
    }

    #[cfg(windows)]
    #[test]
    fn m2_native_owner_outcome_admission_failure_has_no_evidence_or_io() {
        use crate::snapshot_v2_durable_transaction::run_native_owner_outcome;
        let store=SqliteAdapterV1::provision(&empty_root()).unwrap();
        let adapter=FaultAdapter{inner:&store,fault:FaultPoint::None,calls:Default::default()};
        let mut invalid=payload(1,vec![]); invalid.digest=[0;32];
        let failure=run_native_owner_outcome(&adapter,&invalid,None,b"vc04.sqlite-durability.r1").unwrap_err();
        assert_eq!(failure.state,None);
        assert_eq!(failure.cause,Failure::SnapshotHashMismatch);
        assert!(adapter.calls.borrow().is_empty());
    }

    #[test]
    fn sqlite_engine_bounds_hostile_records_and_accepts_maximum_field_sizes() {
        let store = store();
        let mut maximum = payload(2, vec![b'p'; 4096]);
        maximum.binding.project = vec![b'a'; 4096];
        maximum.binding.new_revision = vec![b'r'; 4096];
        maximum.binding.execution_profile = vec![b'e'; 4096];
        maximum.snapshot = vec![0; 2_000_000];
        maximum.binding.snapshot_length = 2_000_000;
        maximum.envelope = vec![0; 1_000_000];
        write(&store, &maximum);
        assert_eq!(
            store.reread_prepared(&maximum.digest).unwrap(),
            Some(maximum)
        );
        let first = payload(1, vec![]);
        write(&store, &first);
        store.commit_verified(&first, None).unwrap();
        let hostile = payload(3, vec![1]);
        write(&store, &hostile);
        let external = Connection::open(&store.path).unwrap();
        external
            .execute_batch("PRAGMA ignore_check_constraints=ON")
            .unwrap();
        external.execute("UPDATE vc04_prepared_payloads SET canonical_snapshot_v2_bytes=zeroblob(?1) WHERE transaction_digest=?2",
            params![MAX_SQLITE_RECORD_BYTES + 1, hostile.digest.as_slice()]).unwrap();
        drop(external);
        let bounded = store.connect().unwrap();
        let read = bounded.query_row("SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE transaction_digest=?1",
            [hostile.digest.as_slice()], |row| row.get::<_,Vec<u8>>(0));
        assert!(
            matches!(read, Err(rusqlite::Error::SqliteFailure(ref error, _))
            if error.extended_code == rusqlite::ffi::SQLITE_TOOBIG)
        );
        assert!(matches!(
            store.reread_prepared(&hostile.digest),
            Err(Failure::RereadFailed)
        ));
        let old = store
            .inspect_recovery(&first.digest, b"project", &[1])
            .unwrap();
        assert_eq!(old.payload, Some(first));
        assert_eq!(old.head, old.revision);
    }
    #[cfg(windows)]
    fn empty_root() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ddn-sqlite-provision-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        root
    }

    #[cfg(windows)]
    #[test]
    fn m2_recovery_child_schema_is_explicit_and_base_writer_cannot_bypass_fence() {
        let root=empty_root();
        let adapter=SqliteAdapterV1::provision_recovery_custody(&root).unwrap();
        assert_eq!(adapter.capability().unwrap(),"vc04.sqlite-durability.r1");
        let candidate=payload(1,vec![]);
        assert!(matches!(adapter.write_prepared(&candidate),Err(Failure::UnsupportedDurability)));
        assert_eq!(adapter.commit_verified(&candidate,None),Err(Failure::UnsupportedDurability));
        for table in ["vc04_prepared_payloads","vc04_project_heads","vc04_project_revisions","vc04_recovery_custody_v1"] {
            let count:i64=adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|row|row.get(0)).unwrap();
            assert_eq!(count,0,"base transaction path wrote {table} without Pending/start/row0");
        }
        drop(adapter);
        assert!(matches!(SqliteAdapterV1::open_admitted(&root),Err(Failure::UnsupportedDurability)));
        let reopened=SqliteAdapterV1::open_recovery_custody(&root).unwrap();
        assert_eq!(reopened.capability().unwrap(),"vc04.sqlite-durability.r1");
        drop(reopened);
        assert!(matches!(SqliteAdapterV1::provision_recovery_custody(&root),Err(Failure::ConflictingRetry)));
        // Explicitly preserve this task-owned container as failure evidence.
    }

    #[cfg(windows)]
    #[test]
    fn m2_recovery_child_schema_never_migrates_base_or_repairs_foreign_schema() {
        let base_root=empty_root();
        let base=SqliteAdapterV1::provision(&base_root).unwrap();
        drop(base);
        assert!(matches!(SqliteAdapterV1::open_recovery_custody(&base_root),Err(Failure::UnsupportedDurability)));
        assert!(matches!(SqliteAdapterV1::provision_recovery_custody(&base_root),Err(Failure::ConflictingRetry)));
        let base=SqliteAdapterV1::open_admitted(&base_root).unwrap();
        let count:i64=base.connect().unwrap().query_row("SELECT count(*) FROM sqlite_schema WHERE name='vc04_recovery_custody_v1'",[],|row|row.get(0)).unwrap();
        assert_eq!(count,0,"child schema was silently added");
        let child_root=empty_root();
        let child=SqliteAdapterV1::provision_recovery_custody(&child_root).unwrap();
        child.connect().unwrap().execute_batch("CREATE TABLE task_owned_foreign_marker(value TEXT)").unwrap();
        drop(child);
        assert!(matches!(SqliteAdapterV1::open_recovery_custody(&child_root),Err(Failure::UnsupportedDurability)));
        // No table/root/WAL deletion or repair is performed by this test.
    }

    #[cfg(windows)]
    #[test]
    fn committed_payload_comparison_requires_actual_post_reopen_confirmation() {
        use crate::snapshot_v2_durable_transaction::TransactionEvidenceV1;
        use sha2::{Digest, Sha256};
        let root = empty_root();
        let adapter = SqliteAdapterV1::provision(&root).unwrap();
        let profile = adapter.capability().unwrap();
        let mut row = payload(1, vec![]);
        row.binding.snapshot_sha256 = Sha256::digest(&row.snapshot).into();
        row.digest = row.binding.transaction_digest().unwrap();
        let mut evidence = TransactionEvidenceV1::prepared(row.binding.clone()).unwrap();
        let mut handle = adapter.write_prepared(&row).unwrap();
        adapter.sync_prepared(&mut handle).unwrap(); evidence.synced().unwrap();
        let prepared = adapter.reread_prepared(&row.digest).unwrap().unwrap();
        assert_eq!(prepared, row);
        evidence.verified(&row.snapshot, &prepared.snapshot).unwrap();
        adapter.commit_verified(&row, None).unwrap(); evidence.head_changed().unwrap();
        // Establish the durable container before normal close. This is not
        // reused as confirmation after reopen: the new handle confirms again.
        adapter.confirm_container_durable().unwrap();
        // Prepared handles retain the root lease as well as the SQL handle.
        // Normal termination must close both, not bypass the exclusive lease.
        drop(handle);
        drop(adapter);
        let reopened = SqliteAdapterV1::open_admitted(&root).unwrap();
        let length = row.binding.snapshot_length.to_be_bytes();
        let fields: [&[u8];12] = [&row.digest, &row.binding.project, &row.binding.expected_parent,
            &row.binding.new_revision, &row.binding.pesi, &row.binding.execution_profile,
            &length, &row.binding.snapshot_sha256, &row.binding.revision_integrity,
            &row.binding.revision_integrity, profile.as_bytes(), &[1]];
        let mut receipt = vec![12];
        for (index, field) in fields.into_iter().enumerate() {
            receipt.push(index as u8 + 1);
            receipt.extend_from_slice(&(field.len() as u32).to_be_bytes());
            receipt.extend_from_slice(field);
        }
        // Exact payload and a visible SQLite head alone cannot confirm it.
        assert!(evidence.check_committed_receipt_evidence(&receipt, profile.as_bytes()).is_err());
        evidence.container_confirmation(reopened.confirm_container_durable()).unwrap();
        let committed = reopened.reread_committed(&row.digest, b"project", &[1]).unwrap();
        assert_eq!(committed.head, committed.revision);
        assert_eq!(committed.payload.as_ref(), Some(&row));
        evidence.committed_native_chain(&row, &committed).unwrap();
        evidence.check_committed_receipt_evidence(&receipt, profile.as_bytes()).unwrap();
        // Mechanics/codec evidence, not admitted source/custody, continuation
        // provenance, a published success receipt, or full Beta1 durability.
    }

    #[cfg(windows)]
    #[test]
    fn common_owner_rejects_divergent_native_chain_and_requires_fresh_confirmation() {
        use crate::snapshot_v2_durable_transaction::{TransactionEvidenceV1,
            SnapshotV2TransactionStateV1 as State};
        use sha2::{Digest, Sha256};
        let adapter = SqliteAdapterV1::provision(&empty_root()).unwrap();
        let mut row = payload(1, vec![]);
        row.binding.snapshot_sha256 = Sha256::digest(&row.snapshot).into();
        row.digest = row.binding.transaction_digest().unwrap();
        write(&adapter, &row);
        adapter.commit_verified(&row, None).unwrap();
        // Three independent risks: mutually matching foreign heads, revision
        // correlation drift, and an envelope substitution beside exact state.
        for mutation in 0..3 {
            let mut evidence = TransactionEvidenceV1::prepared(row.binding.clone()).unwrap();
            evidence.synced().unwrap();
            evidence.verified(&row.snapshot, &row.snapshot).unwrap();
            evidence.head_changed().unwrap();
            evidence.container_confirmation(adapter.confirm_container_durable()).unwrap();
            let mut bad = adapter.reread_committed(&row.digest, b"project", &[1]).unwrap();
            match mutation {
                0 => {
                    bad.head.as_mut().unwrap().project = b"foreign".to_vec();
                    bad.revision.as_mut().unwrap().project = b"foreign".to_vec();
                }
                1 => bad.revision.as_mut().unwrap().digest[0] ^= 1,
                _ => bad.payload.as_mut().unwrap().envelope[0] ^= 1,
            }
            assert!(evidence.committed_native_chain(&row, &bad).is_err());
            assert_eq!(evidence.state(), State::RecoveryRequired);
            let restored = adapter.reread_committed(&row.digest, b"project", &[1]).unwrap();
            assert_eq!(restored.payload.as_ref(), Some(&row));
            assert_eq!(evidence.committed_native_chain(&row, &restored), Err(Failure::RecoveryRequired));
            evidence.container_confirmation(adapter.confirm_container_durable()).unwrap();
            evidence.committed_native_chain(&row, &restored).unwrap();
            assert_eq!(evidence.state(), State::Committed);
        }
        // Opaque SQLite mechanics fixtures prove only the common comparison;
        // they are not full source/custody or RNG continuation admission.
    }

    #[cfg(windows)]
    #[test]
    fn admitted_native_root_provisions_reopens_and_confirms_container() {
        let root = empty_root();
        let adapter = SqliteAdapterV1::provision(&root).unwrap();
        assert_eq!(adapter.capability().unwrap(), "vc04.sqlite-durability.r1");
        assert!(matches!(
            SqliteAdapterV1::open_admitted(&root),
            Err(Failure::UnsupportedDurability)
        ));
        let row = payload(1, vec![]);
        write(&adapter, &row);
        adapter.commit_verified(&row, None).unwrap();
        adapter.confirm_container_durable().unwrap();
        let expected = adapter
            .inspect_recovery(&row.digest, b"project", &[1])
            .unwrap();
        drop(adapter);
        assert!(root.join("vc04_snapshot_v2.sqlite3-wal").is_file());
        assert!(root.join("vc04_snapshot_v2.sqlite3-shm").is_file());
        // Provisioning is never a migration or an overwrite operation.
        assert!(matches!(
            SqliteAdapterV1::provision(&root),
            Err(Failure::ConflictingRetry)
        ));
        let reopened = SqliteAdapterV1::open_admitted(&root).unwrap();
        reopened.confirm_container_durable().unwrap();
        let actual = reopened
            .inspect_recovery(&row.digest, b"project", &[1])
            .unwrap();
        assert_eq!(actual.payload, expected.payload);
        assert_eq!(actual.revision, expected.revision);
        assert_eq!(actual.head, expected.head);
        // Opaque mechanics test values above do not issue a semantic receipt.
    }

    #[cfg(windows)]
    #[test]
    fn admitted_root_lease_excludes_another_process() {
        const PROBE: &str = "DDN_SQLITE_ROOT_LEASE_TEST";
        if let Some(root) = std::env::var_os(PROBE) {
            assert!(matches!(
                SqliteAdapterV1::open_admitted(Path::new(&root)),
                Err(Failure::UnsupportedDurability)
            ));
            return;
        }
        let root = empty_root();
        let adapter = SqliteAdapterV1::provision(&root).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact").arg("snapshot_v2_durable_transaction::native::tests::admitted_root_lease_excludes_another_process")
            .env(PROBE, &root).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(adapter.capability().unwrap(), "vc04.sqlite-durability.r1");
        drop(adapter);
        assert!(SqliteAdapterV1::open_admitted(&root).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn root_admission_refuses_relative_roots_and_never_repairs_unknown_schema() {
        assert!(matches!(
            SqliteAdapterV1::provision(Path::new(".")),
            Err(Failure::UnsupportedDurability)
        ));
        let root = empty_root();
        let adapter = SqliteAdapterV1::provision(&root).unwrap();
        let path = adapter.path.clone();
        drop(adapter);
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch("CREATE TABLE foreign_data(value TEXT)")
            .unwrap();
        drop(connection);
        assert!(matches!(
            SqliteAdapterV1::open_admitted(&root),
            Err(Failure::UnsupportedDurability)
        ));
        let connection = Connection::open(&path).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='foreign_data'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[cfg(windows)]
    #[test]
    fn prepared_handle_keeps_root_lease_and_cannot_be_used_on_another_database() {
        let root = empty_root();
        let adapter = SqliteAdapterV1::provision(&root).unwrap();
        let other = SqliteAdapterV1::provision(&empty_root()).unwrap();
        let mut handle = adapter.write_prepared(&payload(1, vec![])).unwrap();
        assert_eq!(other.sync_prepared(&mut handle), Err(Failure::InvalidInput));
        assert_eq!(other.abort(&mut handle), Err(Failure::InvalidInput));
        drop(adapter);
        // Closing the adapter cannot release coordination while a prepared
        // connection still exists. Drop rolls back before releasing the lease.
        assert!(matches!(
            SqliteAdapterV1::open_admitted(&root),
            Err(Failure::UnsupportedDurability)
        ));
        drop(handle);
        let reopened = SqliteAdapterV1::open_admitted(&root).unwrap();
        assert!(reopened.reread_prepared(&[1; 32]).unwrap().is_none());
    }

    #[cfg(windows)]
    #[test]
    fn abrupt_process_exit_preserves_exact_old_or_new_sqlite_chain() {
        const ROOT: &str = "DDN_SQLITE_INTERRUPTION_ROOT";
        const PHASE: &str = "DDN_SQLITE_INTERRUPTION_PHASE";
        if let Some(root) = std::env::var_os(ROOT) {
            let adapter = SqliteAdapterV1::open_admitted(Path::new(&root)).unwrap();
            let old = adapter
                .inspect_recovery(&[1; 32], b"project", &[1])
                .unwrap()
                .head
                .unwrap();
            let second = payload(2, vec![1]);
            let mut handle = adapter.write_prepared(&second).unwrap();
            let phase = std::env::var(PHASE).unwrap();
            if phase == "before_prepared_commit" {
                std::process::exit(0);
            }
            adapter.sync_prepared(&mut handle).unwrap();
            if phase == "after_prepared_commit" {
                std::process::exit(0);
            }
            assert_eq!(phase, "after_head_commit");
            adapter.commit_verified(&second, Some(&old)).unwrap();
            // Deliberately skip Drop, checkpoint confirmation and any receipt.
            std::process::exit(0);
        }
        for phase in [
            "before_prepared_commit",
            "after_prepared_commit",
            "after_head_commit",
        ] {
            let root = empty_root();
            let first = payload(1, vec![]);
            let adapter = SqliteAdapterV1::provision(&root).unwrap();
            write(&adapter, &first);
            adapter.commit_verified(&first, None).unwrap();
            adapter.confirm_container_durable().unwrap();
            drop(adapter);
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("snapshot_v2_durable_transaction::native::tests::abrupt_process_exit_preserves_exact_old_or_new_sqlite_chain")
                .env(ROOT, &root).env(PHASE, phase).output().unwrap();
            assert!(
                child.status.success(),
                "{phase}: {}",
                String::from_utf8_lossy(&child.stderr)
            );
            let reopened = SqliteAdapterV1::open_admitted(&root).unwrap();
            let second = payload(2, vec![1]);
            let current = reopened
                .inspect_recovery(&second.digest, b"project", &[2])
                .unwrap();
            let old = reopened
                .inspect_recovery(&first.digest, b"project", &[1])
                .unwrap();
            assert_eq!(old.payload, Some(first.clone()));
            assert_eq!(old.revision.as_ref().unwrap().revision, vec![1]);
            if phase == "after_head_commit" {
                assert_eq!(current.payload, Some(second));
                assert_eq!(current.head, current.revision);
                assert_eq!(current.head.unwrap().revision, vec![2]);
            } else {
                assert!(current.revision.is_none());
                assert_eq!(current.head, old.revision);
                if phase == "before_prepared_commit" {
                    assert!(current.payload.is_none());
                } else {
                    assert_eq!(current.payload, Some(second));
                }
            }
            // Raw NEW_COMPLETE-looking rows do not mint a durable receipt.
            reopened.confirm_container_durable().unwrap();
        }
    }

    fn store() -> SqliteAdapterV1 {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "ddn-sqlite-mechanics-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("vc04_snapshot_v2.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(&format!(
                "PRAGMA journal_mode=WAL; PRAGMA user_version=1; {PAYLOAD}; {REVISIONS}; {HEADS};"
            ))
            .unwrap();
        drop(connection);
        // Mechanics tests deliberately do NOT mint a root/custody/receipt proof.
        SqliteAdapterV1::open_existing(&path).unwrap()
    }

    #[test]
    fn vc04_source_profile_reopens_exactly_and_does_not_alias_existing_profiles() {
        let base=store();
        let path=base.path.with_file_name("source-profile-only.sqlite3");
        let db=Connection::open(&path).unwrap();
        db.execute_batch(&format!("PRAGMA journal_mode=WAL; PRAGMA user_version=2; {}; {}; {}; {RECOVERY_CUSTODY};",
            source_revision_schema::PAYLOAD,source_revision_schema::REVISIONS,source_revision_schema::HEADS)).unwrap();
        drop(db);
        let adapter=SqliteAdapterV1 {path:path.clone(),schema_profile:NativeSchemaProfile::SourceRevision,
            root_lease:None,#[cfg(windows)] pinned_root:None,
            #[cfg(windows)] fresh_container:std::cell::RefCell::new(None)};
        adapter.connect().unwrap(); // Exact actual SQLite schema, no authority.
        assert!(matches!(SqliteAdapterV1::open_existing(&path),Err(Failure::UnsupportedDurability)));
        assert!(matches!(adapter.capability(),Err(Failure::UnsupportedDurability)));
        let wrong=SqliteAdapterV1 {path:base.path.clone(),schema_profile:NativeSchemaProfile::SourceRevision,
            root_lease:None,#[cfg(windows)] pinned_root:None,
            #[cfg(windows)] fresh_container:std::cell::RefCell::new(None)};
        assert!(matches!(wrong.connect(),Err(Failure::UnsupportedDurability)));
        base.connect().unwrap();
    }
    fn payload(revision: u8, parent: Vec<u8>) -> StoredPayloadV1 {
        StoredPayloadV1 {
            digest: [revision; 32],
            binding: TransactionBindingV1 {
                project: b"project".to_vec(),
                expected_parent: parent,
                new_revision: vec![revision],
                execution_profile: b"profile".to_vec(),
                pesi: [3; 32],
                snapshot_sha256: [4; 32],
                snapshot_length: 4,
                revision_integrity: [revision; 32],
            },
            snapshot: b"data".to_vec(),
            envelope: b"envelope".to_vec(),
        }
    }
    fn write(store: &SqliteAdapterV1, row: &StoredPayloadV1) {
        let mut handle = store.write_prepared(row).unwrap();
        store.sync_prepared(&mut handle).unwrap();
    }
    #[test]
    fn prepared_fresh_reread_is_exact_and_retry_cannot_replace_bytes() {
        let store = store();
        let row = payload(1, vec![]);
        write(&store, &row);
        assert_eq!(
            store.reread_prepared(&row.digest).unwrap(),
            Some(row.clone())
        );
        write(&store, &row);
        let mut changed = row.clone();
        changed.envelope = b"changed".to_vec();
        assert!(matches!(
            store.write_prepared(&changed),
            Err(Failure::ConflictingRetry)
        ));
        assert_eq!(store.reread_prepared(&row.digest).unwrap(), Some(row));
    }
    #[test]
    fn revision_and_full_head_cas_commit_together_and_stale_parent_rolls_back() {
        let store = store();
        let first = payload(1, vec![]);
        write(&store, &first);
        store.commit_verified(&first, None).unwrap();
        let old = store
            .reread_committed(&first.digest, b"project", &[1])
            .unwrap()
            .head
            .unwrap();
        let second = payload(2, vec![1]);
        write(&store, &second);
        store.commit_verified(&second, Some(&old)).unwrap();
        let chain = store
            .inspect_recovery(&second.digest, b"project", &[2])
            .unwrap();
        assert_eq!(chain.payload, Some(second.clone()));
        assert_eq!(chain.head, chain.revision);
        let third = payload(3, vec![1]);
        write(&store, &third);
        assert_eq!(
            store.commit_verified(&third, Some(&old)),
            Err(Failure::StaleRevision)
        );
        let chain = store
            .reread_committed(&third.digest, b"project", &[3])
            .unwrap();
        assert!(chain.revision.is_none());
        assert_eq!(chain.head.unwrap().revision, vec![2]);
    }
    #[test]
    fn abort_cannot_publish_and_profile_downgrade_is_not_repaired() {
        let store = store();
        let row = payload(1, vec![]);
        let mut handle = store.write_prepared(&row).unwrap();
        store.abort(&mut handle).unwrap();
        assert!(store.reread_prepared(&row.digest).unwrap().is_none());
        let connection = Connection::open(&store.path).unwrap();
        connection
            .execute_batch("PRAGMA journal_mode=DELETE")
            .unwrap();
        drop(connection);
        assert!(matches!(
            SqliteAdapterV1::open_existing(&store.path),
            Err(Failure::UnsupportedDurability)
        ));
        let connection = Connection::open(&store.path).unwrap();
        let mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "delete");
    }
    #[test]
    fn unknown_schema_is_rejected_before_write() {
        let store = store();
        let connection = Connection::open(&store.path).unwrap();
        connection
            .execute_batch("CREATE TABLE foreign_data(value TEXT)")
            .unwrap();
        drop(connection);
        assert!(matches!(
            store.write_prepared(&payload(1, vec![])),
            Err(Failure::UnsupportedDurability)
        ));
    }
    #[test]
    fn mechanical_commit_without_admitted_root_cannot_claim_durability() {
        let store = store();
        let row = payload(1, vec![]);
        write(&store, &row);
        store.commit_verified(&row, None).unwrap();
        assert_eq!(store.capability(), Err(Failure::UnsupportedDurability));
        assert_eq!(
            store.confirm_container_durable(),
            Err(Failure::UnsupportedDurability)
        );
        let chain = store
            .inspect_recovery(&row.digest, b"project", &[1])
            .unwrap();
        assert_eq!(chain.payload, Some(row));
        // This is raw NEW-looking data, not a Committed receipt/recovery verdict.
        assert_eq!(chain.head, chain.revision);
    }
    #[test]
    fn oversized_payload_is_rejected_before_database_discovery() {
        let existing = store();
        let missing = SqliteAdapterV1 {
            path: existing.path.with_file_name("never-created.sqlite3"),
            schema_profile:NativeSchemaProfile::Base,
            root_lease: None,
            #[cfg(windows)] pinned_root: None,
            #[cfg(windows)] fresh_container: std::cell::RefCell::new(None),
        };
        let mut row = payload(1, vec![]);
        row.snapshot.resize(2_000_001, 0);
        row.binding.snapshot_length = row.snapshot.len() as u64;
        assert!(matches!(
            missing.write_prepared(&row),
            Err(Failure::LimitExceeded)
        ));
        assert!(!missing.path.exists());
        row = payload(1, vec![]);
        row.envelope.resize(1_000_001, 0);
        assert!(matches!(
            missing.write_prepared(&row),
            Err(Failure::LimitExceeded)
        ));
        assert!(!missing.path.exists());
    }
    #[test]
    fn expected_head_integrity_and_prepared_envelope_are_not_ignored() {
        let store = store();
        let first = payload(1, vec![]);
        write(&store, &first);
        store.commit_verified(&first, None).unwrap();
        let old = store
            .inspect_recovery(&first.digest, b"project", &[1])
            .unwrap()
            .head
            .unwrap();
        let second = payload(2, vec![1]);
        write(&store, &second);
        let mut wrong = old.clone();
        wrong.integrity[0] ^= 1;
        assert_eq!(
            store.commit_verified(&second, Some(&wrong)),
            Err(Failure::StaleRevision)
        );
        let mut changed = second.clone();
        changed.envelope = b"other-envelope".to_vec();
        assert_eq!(
            store.commit_verified(&changed, Some(&old)),
            Err(Failure::RereadMismatch)
        );
        let chain = store
            .reread_committed(&second.digest, b"project", &[2])
            .unwrap();
        assert!(chain.revision.is_none());
        assert_eq!(chain.head, Some(old.clone()));
        store.commit_verified(&second, Some(&old)).unwrap();
        let chain = store
            .reread_committed(&second.digest, b"project", &[2])
            .unwrap();
        assert_eq!(chain.head, chain.revision);
    }

    #[test]
    fn corrupt_persisted_payload_is_refused_without_replacing_old_head() {
        let store = store();
        let first = payload(1, vec![]);
        write(&store, &first);
        store.commit_verified(&first, None).unwrap();
        let old = store
            .inspect_recovery(&first.digest, b"project", &[1])
            .unwrap();
        let second = payload(2, vec![1]);
        write(&store, &second);
        let connection = Connection::open(&store.path).unwrap();
        // An external writer can bypass CHECK; the read path must not rely on
        // creation-time schema validation as proof that row values are bounded.
        connection
            .execute_batch("PRAGMA ignore_check_constraints=ON")
            .unwrap();
        for statement in [
            "UPDATE vc04_prepared_payloads SET snapshot_v2_byte_length_u64=-1 WHERE transaction_digest=?1",
            "UPDATE vc04_prepared_payloads SET snapshot_v2_byte_length_u64=4,canonical_snapshot_v2_bytes=zeroblob(2000001) WHERE transaction_digest=?1",
            "UPDATE vc04_prepared_payloads SET canonical_snapshot_v2_bytes=x'64617461',revision_envelope_bytes=zeroblob(1000001) WHERE transaction_digest=?1",
            "UPDATE vc04_prepared_payloads SET revision_envelope_bytes=x'656e76656c6f7065',project_identity=zeroblob(4097) WHERE transaction_digest=?1",
        ] {
            connection.execute(statement, [second.digest.as_slice()]).unwrap();
            assert!(matches!(store.reread_prepared(&second.digest), Err(Failure::RereadFailed)));
            let intact = store.inspect_recovery(&first.digest, b"project", &[1]).unwrap();
            assert_eq!(intact.payload, old.payload);
            assert_eq!(intact.revision, old.revision);
            assert_eq!(intact.head, old.head);
        }
    }
}
