//! Source-only initial save and exact cold readback. These operations never
//! issue a RootGrant first-use proof, run a VM, or post a world receipt.
use super::{SqliteAdapterV1,NativeSchemaProfile,Failure,database_error};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
use rusqlite::{params,types::ValueRef,OptionalExtension};

#[derive(Debug)]
pub(crate) enum SourceSaveFailure { BeforeCommit(Failure), AfterCommitUnconfirmed(Failure) }
pub(crate) enum SourceEditV2 {Unchanged(Source),Successor(Source)}
pub(crate) struct DurablySavedSourceRevision<'a> { _lease:&'a SqliteAdapterV1,source:Source }
impl DurablySavedSourceRevision<'_> {pub(crate) fn source(&self)->&Source {&self.source}}
#[cfg(test)]
std::thread_local! {static SAVE_FAULT:std::cell::Cell<u8>=const{std::cell::Cell::new(0)};}
#[cfg(test)]
pub(super) fn inject_save_fault(phase:u8){assert!(phase<=2);SAVE_FAULT.with(|fault|fault.set(phase));}

impl SqliteAdapterV1 {
    /// Distinct W/S -> edited S entry. Only the actual protected authoring
    /// producer constructs this token; the old Unused source entry is intact.
    #[cfg(windows)]
    pub(crate) fn save_authoring_source(&mut self,prepared:super::PreparedAuthoringSourceV1)
        ->Result<super::DurablySavedAuthoringSourceV1,SourceSaveFailure>{
        let _observation = crate::storage_observation::Span::start("sqlite.source_revision.save_authoring_total");
        use SourceSaveFailure::{BeforeCommit as Before,AfterCommitUnconfirmed as After};
        if self.schema_profile!=NativeSchemaProfile::SourceRevision{return Err(Before(Failure::UnsupportedDurability));}
        prepared.verify_before_save(self).map_err(Before)?;
        let source=prepared.source();let parent=prepared.parent();
        let db=self.connect().map_err(Before)?;db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Before(Failure::WriteFailed))?;
        let write=(||->Result<(),Failure>{
            let count:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_project_revisions LIMIT 65537)",[],|r|r.get(0)).map_err(database_error)?;
            if count>=65536{return Err(Failure::LimitExceeded);}
            db.execute("INSERT INTO vc04_prepared_payloads VALUES (0,?1,NULL,?2,?3,?4,?5,?6,NULL,NULL,NULL,?7,?1,?8)",
                params![source.integrity().as_slice(),source.project(),source.parent(),source.revision(),source.profile(),source.pesi().as_slice(),source.payload(),source.envelope()]).map_err(|_|Failure::WriteFailed)?;
            db.execute("INSERT INTO vc04_project_revisions VALUES(?1,?2,?3,0,?3)",params![source.project(),source.revision(),source.integrity().as_slice()]).map_err(|_|Failure::WriteFailed)?;
            let changes=db.execute("UPDATE vc04_project_heads SET history_revision_identity=?1,revision_integrity_sha256=?2,payload_kind=0,payload_key=?2
                WHERE project_identity=?3 AND history_revision_identity=?4 AND revision_integrity_sha256=?5 AND payload_kind=?6 AND payload_key=?7",
                params![source.revision(),source.integrity().as_slice(),source.project(),parent.revision,parent.integrity.as_slice(),parent.kind,parent.key.as_slice()]).map_err(|_|Failure::WriteFailed)?;
            if changes!=1{return Err(Failure::StaleRevision);}Ok(())
        })();
        if let Err(error)=write{return match db.execute_batch("ROLLBACK"){Ok(())=>Err(Before(error)),Err(_)=>Err(After(Failure::RecoveryRequired))};}
        let authoring_commit_observation = crate::storage_observation::Span::start("sqlite.source_revision.authoring_full_commit_and_close");
        db.execute_batch("COMMIT").map_err(|_|After(Failure::WriteFailed))?;drop(db);
        drop(authoring_commit_observation);
        prepared.confirm_saved(self).map_err(After)
    }
    #[cfg(windows)]
    pub(crate) fn save_source_successor<'a>(&'a mut self,source:&Source,
        grant:&super::root_grant::AdmittedSourceGrantV2)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure>{
        let _observation = crate::storage_observation::Span::start("sqlite.source_revision.save_successor_total");
        use SourceSaveFailure::{BeforeCommit as Before,AfterCommitUnconfirmed as After};
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||source.parent().is_empty(){return Err(Before(Failure::UnsupportedDurability));}
        let expected_integrity=grant.verify_source_successor(self,source).map_err(Before)?;
        self.commit_source_successor(source,&expected_integrity)?;
        grant.verify_saved_source(self,source).map_err(After)?;
        let chain=self.read_source_chain(source.project()).map_err(After)?;
        if chain.latest.revision()!=source.revision()||chain.latest.integrity()!=source.integrity(){return Err(After(Failure::RereadMismatch));}
        Ok(DurablySavedSourceRevision{_lease:self,source:chain.latest})
    }

    #[cfg(windows)]
    fn commit_source_successor(&mut self,source:&Source,expected_integrity:&[u8;32])->Result<(),SourceSaveFailure>{
        use SourceSaveFailure::{BeforeCommit as Before,AfterCommitUnconfirmed as After};
        let db=self.connect().map_err(Before)?;db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Before(Failure::WriteFailed))?;
        let write=(||->Result<(),Failure>{
            let count:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_project_revisions LIMIT 65536)",[],|r|r.get(0)).map_err(database_error)?;
            if count>=65536{return Err(Failure::LimitExceeded);}
            db.execute("INSERT INTO vc04_prepared_payloads VALUES (0,?1,NULL,?2,?3,?4,?5,?6,NULL,NULL,NULL,?7,?1,?8)",
                params![source.integrity().as_slice(),source.project(),source.parent(),source.revision(),source.profile(),source.pesi().as_slice(),source.payload(),source.envelope()]).map_err(|_|Failure::WriteFailed)?;
            db.execute("INSERT INTO vc04_project_revisions VALUES(?1,?2,?3,0,?3)",
                params![source.project(),source.revision(),source.integrity().as_slice()]).map_err(|_|Failure::WriteFailed)?;
            #[cfg(test)]
            if SAVE_FAULT.with(|f|f.get())==1{return Err(Failure::WriteFailed);}
            let changes=db.execute("UPDATE vc04_project_heads SET history_revision_identity=?1,revision_integrity_sha256=?2,payload_kind=0,payload_key=?2
                WHERE project_identity=?3 AND history_revision_identity=?4 AND payload_kind=0 AND revision_integrity_sha256=?5 AND payload_key=?5",
                params![source.revision(),source.integrity().as_slice(),source.project(),source.parent(),expected_integrity.as_slice()]).map_err(|_|Failure::WriteFailed)?;
            if changes!=1{return Err(Failure::StaleRevision);}Ok(())
        })();
        if let Err(error)=write{return match db.execute_batch("ROLLBACK"){Ok(())=>Err(Before(error)),Err(_)=>Err(After(Failure::RecoveryRequired))};}
        let successor_commit_observation = crate::storage_observation::Span::start("sqlite.source_revision.successor_full_commit_and_close");
        db.execute_batch("COMMIT").map_err(|_|After(Failure::WriteFailed))?;drop(db);
        drop(successor_commit_observation);
        #[cfg(test)]
        if SAVE_FAULT.with(|f|f.get())==2{return Err(After(Failure::RereadFailed));}
        Ok(())
    }

    /// Append the first spelling-only edition to the closed source-only
    /// store, using its existing parent CAS and protected root lease.
    #[cfg(windows)]
    pub(crate) fn save_public_edition_successor(&mut self,previous:&Source,source:&Source)->Result<Source,SourceSaveFailure>{
        use SourceSaveFailure::{BeforeCommit as Before,AfterCommitUnconfirmed as After};
        let actual=self.load_current_source_revision(previous.project(),previous.revision(),previous.integrity()).map_err(Before)?;
        if actual.source().artifact().map_err(|_|Before(Failure::CustodyConflict))?!=previous.artifact().map_err(|_|Before(Failure::CustodyConflict))?
            ||source.project()!=previous.project()||source.parent()!=previous.revision(){
            return Err(Before(Failure::CustodyConflict));
        }
        let checked=Source::restore_public_edition(actual.source(),source.revision(),source.integrity(),
            source.profile(),source.pesi(),source.payload(),source.envelope())
            .map_err(|_|Before(Failure::CustodyConflict))?;
        self.commit_source_successor(&checked,previous.integrity())?;
        self.read_public_edition_history(previous,checked.revision(),checked.integrity())
            .map(|(_,child)|child).map_err(After)
    }

    #[cfg(windows)]
    pub(crate) fn read_public_edition_history(&self,previous:&Source,revision:&[u8],integrity:&[u8;32])->Result<(Source,Source),Failure>{
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||!previous.parent().is_empty()
            ||revision.is_empty()||revision.len()>4096||revision==previous.revision(){return Err(Failure::CustodyConflict);}
        self.confirm_container_durable()?;
        let db=self.connect()?;
        db.execute_batch("BEGIN").map_err(database_error)?;
        for (table,expected) in [("vc04_project_heads",1),("vc04_project_revisions",2),("vc04_prepared_payloads",2),("vc04_recovery_custody_v1",0)]{
            let count:i64=db.query_row(&format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 3)"),[],|row|row.get(0)).map_err(database_error)?;
            if count!=expected{return Err(Failure::CustodyConflict);}
        }
        let head:i64=db.query_row("SELECT count(*) FROM vc04_project_heads WHERE project_identity=?1 AND history_revision_identity=?2
            AND revision_integrity_sha256=?3 AND payload_kind=0 AND payload_key=?3",
            params![previous.project(),revision,integrity.as_slice()],|row|row.get(0)).map_err(database_error)?;
        if head!=1{return Err(Failure::StaleRevision);}
        let old=super::source_chain::read(&db,previous.project(),previous.revision())?;
        if old.integrity()!=previous.integrity()||!old.parent().is_empty(){return Err(Failure::CustodyConflict);}
        let new=super::source_chain::read_public_edition(&db,&old,revision)?;
        if new.integrity()!=integrity{return Err(Failure::RereadMismatch);}
        db.execute_batch("COMMIT").map_err(database_error)?;
        Ok((old,new))
    }

    // The workbench locator anchors the original edition. The protected store,
    // rather than a second receipt file, supplies the successor after restart.
    #[cfg(windows)]
    pub(crate) fn read_public_editions_at_anchor(
        &self, project:&[u8], revision:&[u8], integrity:&[u8;32],
    )->Result<(Source,Option<Source>),Failure>{
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||project.len()!=32
            ||revision.is_empty()||revision.len()>4096{
            return Err(Failure::UnsupportedDurability);
        }
        self.confirm_container_durable()?;
        let db=self.connect()?;
        let (head_revision,head_integrity)=db.query_row(
            "SELECT history_revision_identity,revision_integrity_sha256 FROM vc04_project_heads
                WHERE project_identity=?1 AND payload_kind=0 AND payload_key=revision_integrity_sha256",
            [project],|row|Ok((row.get::<_,Vec<u8>>(0)?,row.get::<_,Vec<u8>>(1)?)),
        ).optional().map_err(database_error)?.ok_or(Failure::RereadMismatch)?;
        drop(db);
        if head_revision==revision{
            if head_integrity!=integrity{return Err(Failure::CustodyConflict);}
            let initial=self.load_current_source_revision(project,revision,integrity)?;
            return Ok((initial.source().clone(),None));
        }
        // The anchor may name only the initial source and the existing
        // history reader must independently confirm both rows and the head.
        let db=self.connect()?;
        let initial=super::source_chain::read(&db,project,revision)?;
        if initial.integrity()!=integrity||!initial.parent().is_empty(){return Err(Failure::CustodyConflict);}
        drop(db);
        let head_integrity=head_integrity.try_into().map_err(|_|Failure::CustodyConflict)?;
        let (initial,head)=self.read_public_edition_history(&initial,&head_revision,&head_integrity)?;
        Ok((initial,Some(head)))
    }

    /// The bounded initial source workflow only. Updating a world parent or
    /// bypassing an unresolved grant/Attempt is not supported by this entry.
    pub(crate) fn save_initial_source_revision<'a>(&'a mut self,source:&Source)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure> {
        self.save_initial_source_revision_bound(source,None)
    }

    pub(super) fn save_initial_source_revision_bound<'a>(&'a mut self,source:&Source,
        grant:Option<&super::root_grant::AdmittedSourceGrantV2>)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure> {
        self.save_initial_source_revision_selected(source,grant,false)
    }
    /// Private codec integration; the existing transaction and CAS are shared.
    pub(crate) fn save_initial_logic_source_revision_private<'a>(&'a mut self,source:&Source)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure>{
        self.save_initial_source_revision_selected(source,None,true)
    }
    #[cfg(windows)]
    pub(super) fn save_initial_logic_source_revision_bound_private<'a>(&'a mut self,source:&Source,
        grant:Option<&super::root_grant::AdmittedSourceGrantV2>)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure>{
        self.save_initial_source_revision_selected(source,grant,true)
    }
    fn save_initial_source_revision_selected<'a>(&'a mut self,source:&Source,grant:Option<&super::root_grant::AdmittedSourceGrantV2>,logic_private:bool)->Result<DurablySavedSourceRevision<'a>,SourceSaveFailure>{
        if logic_private {
            source.owned().registration().ok_or(SourceSaveFailure::BeforeCommit(Failure::CustodyConflict))?
                .verify_production_logic_dependency(&source.owned().precanon_view()).map_err(|_|SourceSaveFailure::BeforeCommit(Failure::CustodyConflict))?;
        }else if source.owned().sources().logic_callable_context().is_some(){return Err(SourceSaveFailure::BeforeCommit(Failure::UnsupportedDurability));}
        let _observation = crate::storage_observation::Span::start("sqlite.source_revision.save_initial_total");
        use SourceSaveFailure::{BeforeCommit as Before,AfterCommitUnconfirmed as After};
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||!source.parent().is_empty() {
            return Err(Before(Failure::UnsupportedDurability));
        }
        // Registration consumed the live fresh-container token. Only its
        // real protected, source-bound Grant2 may replace that token here;
        // the legacy None branch still requires the original fresh proof.
        if grant.is_none(){self.verify_registration_expected_head(source.project(),None).map_err(Before)?;}
        let db=self.connect().map_err(Before)?;
        #[cfg(windows)]
        match grant {
            Some(grant)=>grant.verify_initial_source(self,source).map_err(Before)?,
            None=>self.fresh_container.borrow().as_ref().ok_or(Before(Failure::CustodyConflict))?
                .verify_source_connection(&self.path,&db).map_err(Before)?,
        }
        db.execute_batch("BEGIN IMMEDIATE").map_err(|_|Before(Failure::WriteFailed))?;
        let write=(||->Result<(),Failure>{
            for table in ["vc04_prepared_payloads","vc04_project_revisions","vc04_project_heads","vc04_recovery_custody_v1"] {
                let count:i64=db.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get(0)).map_err(database_error)?;
                if count!=0{return Err(Failure::StaleRevision);}
            }
            db.execute("INSERT INTO vc04_prepared_payloads VALUES (0,?1,NULL,?2,NULL,?3,?4,?5,NULL,NULL,NULL,?6,?1,?7)",
                params![source.integrity().as_slice(),source.project(),source.revision(),source.profile(),source.pesi().as_slice(),source.payload(),source.envelope()]).map_err(|_|Failure::WriteFailed)?;
            db.execute("INSERT INTO vc04_project_revisions VALUES(?1,?2,?3,0,?3)",
                params![source.project(),source.revision(),source.integrity().as_slice()]).map_err(|_|Failure::WriteFailed)?;
            #[cfg(test)]
            if SAVE_FAULT.with(|f|f.get())==1{return Err(Failure::WriteFailed);}
            if db.execute("INSERT INTO vc04_project_heads VALUES(?1,?2,?3,0,?3)",
                params![source.project(),source.revision(),source.integrity().as_slice()]).map_err(|_|Failure::WriteFailed)?!=1 {
                return Err(Failure::StaleRevision);
            }
            Ok(())
        })();
        if let Err(error)=write {return match db.execute_batch("ROLLBACK") {Ok(())=>Err(Before(error)),Err(_)=>Err(After(Failure::RecoveryRequired))};}
        // COMMIT can have reached disk even when its acknowledgement fails.
        let initial_commit_observation = crate::storage_observation::Span::start("sqlite.source_revision.initial_full_commit");
        db.execute_batch("COMMIT").map_err(|_|After(Failure::WriteFailed))?;
        drop(initial_commit_observation);
        drop(db);
        #[cfg(test)]
        if SAVE_FAULT.with(|f|f.get())==2{return Err(After(Failure::RereadFailed));}
        self.load_current_source_revision_selected(source.project(),source.revision(),source.integrity(),logic_private).map_err(After)
    }

    /// Actual new connection + full raw reconstruction under the OS root
    /// lease. This proves source readback only, not cold first-world admission.
    pub(crate) fn load_current_source_revision<'a>(&'a self,project:&[u8],revision:&[u8],integrity:&[u8;32])
        ->Result<DurablySavedSourceRevision<'a>,Failure> {
        self.load_current_source_revision_selected(project,revision,integrity,false)
    }
    pub(crate) fn load_current_logic_source_revision_private<'a>(&'a self,project:&[u8],revision:&[u8],integrity:&[u8;32])->Result<DurablySavedSourceRevision<'a>,Failure>{
        self.load_current_source_revision_selected(project,revision,integrity,true)
    }
    fn load_current_source_revision_selected<'a>(&'a self,project:&[u8],revision:&[u8],integrity:&[u8;32],logic_private:bool)->Result<DurablySavedSourceRevision<'a>,Failure>{
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||project.len()!=32||revision.is_empty()||revision.len()>4096 {
            return Err(Failure::UnsupportedDurability);
        }
        self.confirm_container_durable()?;
        let db=self.connect()?;
        for (table,expected) in [("vc04_project_heads",1),("vc04_project_revisions",1),("vc04_prepared_payloads",1),("vc04_recovery_custody_v1",0)] {
            let count:i64=db.query_row(&format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 2)"),[],|r|r.get(0)).map_err(database_error)?;
            if count!=expected{return Err(Failure::UnsupportedDurability);}
        }
        let source=db.query_row("SELECT h.history_revision_identity,h.revision_integrity_sha256,h.payload_kind,h.payload_key,
            r.revision_integrity_sha256,r.payload_kind,r.payload_key,
            p.project_identity,p.expected_parent_history_revision_identity,p.new_history_revision_identity,
            p.execution_profile_identity,p.project_execution_snapshot_identity,p.revision_integrity_sha256,
            p.revision_envelope_bytes,p.project_payload_bytes,p.payload_kind,p.payload_key,
            p.transaction_digest,p.canonical_snapshot_v2_bytes,p.snapshot_v2_byte_length_u64,p.snapshot_v2_sha256
            FROM vc04_project_heads h JOIN vc04_project_revisions r ON h.project_identity=r.project_identity AND h.history_revision_identity=r.history_revision_identity
            JOIN vc04_prepared_payloads p ON r.payload_kind=p.payload_kind AND r.payload_key=p.payload_key WHERE h.project_identity=?1",
            [project],|row| {
                let blob=|index,limit|->rusqlite::Result<&[u8]>{match row.get_ref(index)? {ValueRef::Blob(v) if !v.is_empty()&&v.len()<=limit=>Ok(v),_=>Err(rusqlite::Error::InvalidQuery)}};
                if blob(0,4096)?!=revision||blob(1,32)?!=integrity||blob(3,32)?!=integrity
                    ||blob(4,32)?!=integrity||blob(6,32)?!=integrity||blob(7,32)?!=project
                    ||blob(9,4096)?!=revision||blob(12,32)?!=integrity||blob(16,32)?!=integrity
                    ||[2,5,15].iter().any(|i|row.get::<_,i64>(*i).ok()!=Some(0))
                    ||[8,17,18,19,20].iter().any(|i|!matches!(row.get_ref(*i),Ok(ValueRef::Null))) {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                let pesi:[u8;32]=blob(11,32)?.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?;
                let restore=if logic_private{Source::restore_logic_private}else{Source::restore};
                restore(project,&[],revision,integrity,blob(10,4096)?,&pesi,blob(14,2_000_000)?,blob(13,1_000_000)?)
                    .map_err(|_|rusqlite::Error::InvalidQuery)
            }).optional().map_err(database_error)?.ok_or(Failure::RereadMismatch)?;
        Ok(DurablySavedSourceRevision{_lease:self,source})
    }
}

#[cfg(all(test,windows))]
mod tests {
    use super::*;
    fn directory()->std::path::PathBuf {
        let mut id=[0;32];getrandom::fill(&mut id).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-source-only-{}",hex::encode(id)));
        std::fs::create_dir(&root).unwrap();root
    }
    fn project()->crate::authored_project::ProjectArtifact {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let body="명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #빨강. }.";
        ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:root.into()},SourceFile{path:"main.ddn".into(),text:body.into()}]},String::new()).unwrap()
    }
    fn source(adapter:&mut SqliteAdapterV1)->Source {
        use crate::authored_project::registration;
        let project=project();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(project.save().unwrap().as_bytes()).unwrap();
        assert!(Source::prepare(&owned,&[],&[2]).is_err(),"schema/atom source without recorded registration must not be admitted");
        let draft=registration::prepare_native_authoring(adapter,&owned,None,None).unwrap().finish().unwrap();
        let owned=owned.bind_registration_candidate(draft).unwrap();
        Source::prepare(&owned,&[],&[2]).unwrap()
    }
    #[test]
    #[ignore="explicit child of vc04_source_only_save_cold_and_raw_corruption"]
    fn vc04_source_only_cold_process_probe() {
        let root=std::env::var("DDN_SOURCE_ONLY_TEST_ROOT").unwrap();
        let project=hex::decode(std::env::var("DDN_SOURCE_ONLY_TEST_PROJECT").unwrap()).unwrap();
        let integrity:[u8;32]=hex::decode(std::env::var("DDN_SOURCE_ONLY_TEST_INTEGRITY").unwrap()).unwrap().try_into().unwrap();
        let adapter=SqliteAdapterV1::open_source_revision(std::path::Path::new(&root)).unwrap();
        let read=adapter.load_current_source_revision(&project,&[2],&integrity).unwrap();
        assert!(read.source().owned().registration().is_some());
        println!("SOURCE_ONLY_COLD_READBACK_OK {}",hex::encode(read.source().integrity()));
    }
    #[test]
    fn vc04_source_only_save_cold_and_raw_corruption() {
        let root=directory();let mut adapter=SqliteAdapterV1::provision_source_revision(&root).unwrap();
        let original=source(&mut adapter);let project=original.project().to_vec();let integrity=*original.integrity();
        let read=adapter.save_initial_source_revision(&original).unwrap();
        assert_eq!(read.source().payload(),original.payload());assert_eq!(read.source().envelope(),original.envelope());
        assert_eq!(read.source().owned().sources().recorded_capture_catalog_json().unwrap(),original.owned().sources().recorded_capture_catalog_json().unwrap());
        drop(read);drop(adapter);
        let child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","snapshot_v2_durable_transaction::native::source_revision::tests::vc04_source_only_cold_process_probe","--ignored","--nocapture"])
            .env("DDN_SOURCE_ONLY_TEST_ROOT",&root).env("DDN_SOURCE_ONLY_TEST_PROJECT",hex::encode(&project))
            .env("DDN_SOURCE_ONLY_TEST_INTEGRITY",hex::encode(integrity))
            .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
        let pid=child.id();let child=child.wait_with_output().unwrap();
        println!("SOURCE_ONLY_COLD_CHILD pid={pid} status={} {}",child.status,String::from_utf8_lossy(&child.stdout));
        assert!(child.status.success(),"{}",String::from_utf8_lossy(&child.stderr));
        assert!(String::from_utf8_lossy(&child.stdout).contains("SOURCE_ONLY_COLD_READBACK_OK"));
        let mut adapter=SqliteAdapterV1::open_source_revision(&root).unwrap();
        assert!(matches!(adapter.save_initial_source_revision(&original),Err(SourceSaveFailure::BeforeCommit(_))),"reopening must not recreate fresh authoring permission");
        let db=adapter.connect().unwrap();
        db.execute("UPDATE vc04_prepared_payloads SET project_payload_bytes=x'7b7d'",[]).unwrap();
        assert!(adapter.load_current_source_revision(&project,&[2],&integrity).is_err(),"actual changed raw bytes must reject");
        db.execute("UPDATE vc04_prepared_payloads SET project_payload_bytes=?1",[original.payload()]).unwrap();drop(db);
        adapter.load_current_source_revision(&project,&[2],&integrity).unwrap();
        println!("SOURCE_ONLY_RETAINED {}",root.display());
    }
    #[test]
    fn vc04_source_only_failure_publication_is_atomic() {
        let root=directory();let mut adapter=SqliteAdapterV1::provision_source_revision(&root).unwrap();let original=source(&mut adapter);
        SAVE_FAULT.with(|f|f.set(1));
        assert!(matches!(adapter.save_initial_source_revision(&original),Err(SourceSaveFailure::BeforeCommit(Failure::WriteFailed))));
        SAVE_FAULT.with(|f|f.set(0));
        for table in ["vc04_prepared_payloads","vc04_project_revisions","vc04_project_heads"] {
            assert_eq!(adapter.connect().unwrap().query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);
        }
        SAVE_FAULT.with(|f|f.set(2));
        assert!(matches!(adapter.save_initial_source_revision(&original),Err(SourceSaveFailure::AfterCommitUnconfirmed(Failure::RereadFailed))));
        SAVE_FAULT.with(|f|f.set(0));
        let read=adapter.load_current_source_revision(original.project(),original.revision(),original.integrity()).unwrap();
        assert_eq!(read.source().payload(),original.payload());
        println!("SOURCE_ONLY_FAILURE_RETAINED {}",root.display());
    }
    #[test]
    fn vc04_source_only_actual_runtime_surface_preserves_raw_source_and_registration() {
        let root=directory();let artifact=project().save().unwrap();
        let saved=crate::runtime_surface::save_initial_authored_source_native(&artifact,&[2],&root).unwrap();
        let read=crate::runtime_surface::read_initial_authored_source_native(&root,&saved.project_identity,&saved.revision_identity,&saved.revision_integrity).unwrap();
        assert_eq!(saved.project_payload_utf8,read.project_payload_utf8);
        assert_eq!(saved.revision_envelope,read.revision_envelope);assert_eq!(saved.pesi,read.pesi);
        let payload:serde_json::Value=serde_json::from_str(&read.project_payload_utf8).unwrap();
        assert_eq!(payload["authored_project_json_utf8"].as_str(),Some(artifact.as_str()));
        assert!(payload["registration_resources"].as_array().unwrap().len()>=2);
        assert!(!payload.as_object().unwrap().contains_key("runtime_state"));
        assert!(crate::runtime_surface::save_initial_authored_source_native(&artifact,&[3],&root).is_err(),"existing DB is not silently migrated or reprovisioned");
        let again=crate::runtime_surface::read_initial_authored_source_native(&root,&saved.project_identity,&saved.revision_identity,&saved.revision_integrity).unwrap();
        assert_eq!(again.project_payload_utf8,saved.project_payload_utf8);
        println!("SOURCE_ONLY_RUNTIME_SURFACE_RETAINED {}",root.display());
    }
}
