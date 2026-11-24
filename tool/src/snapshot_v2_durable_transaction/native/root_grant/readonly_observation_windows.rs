// Private observation: no conversion to AdmittedAuthoringBase/Start/Committed.
// Existing execution and durable-container paths never call this producer.
struct ReadonlySourceFiles {
    _root:std::fs::File,
    files:Vec<std::fs::File>,
    digests:Vec<[u8;32]>,
}
impl ReadonlySourceFiles {
    fn digest(file:&mut std::fs::File)->Result<[u8;32],Failure>{
        use std::io::{Read,Seek,SeekFrom};use sha2::{Digest,Sha256};
        file.seek(SeekFrom::Start(0)).map_err(|_|Failure::RereadFailed)?;
        let mut hash=Sha256::new();let mut buffer=[0u8;65536];
        loop {let n=file.read(&mut buffer).map_err(|_|Failure::RereadFailed)?;if n==0{break}hash.update(&buffer[..n]);}
        Ok(hash.finalize().into())
    }
    fn open(root:&std::path::Path)->Result<Self,Failure>{
        use std::os::windows::fs::OpenOptionsExt;
        if !root.is_absolute(){return Err(Failure::UnsupportedDurability);}
        // Same fixed-local-drive boundary as existing Native admission. Reject
        // UNC/mapped remote roots before any metadata/open can contact a share.
        let drive=match root.components().next(){
            Some(std::path::Component::Prefix(p))=>match p.kind(){
                std::path::Prefix::Disk(d)|std::path::Prefix::VerbatimDisk(d)=>d,
                _=>return Err(Failure::UnsupportedDurability),
            },_=>return Err(Failure::UnsupportedDurability),
        };
        #[link(name="kernel32")] unsafe extern "system" {fn GetDriveTypeW(path:*const u16)->u32;}
        let drive_root=[drive as u16,b':' as u16,b'\\' as u16,0];
        if unsafe{GetDriveTypeW(drive_root.as_ptr())}!=3{return Err(Failure::UnsupportedDurability);}
        for ancestor in root.ancestors(){if !ancestor.as_os_str().is_empty(){super::super::reject_reparse(ancestor)?;}}
        // No WRITE or DELETE sharing: existing/concurrent writer or mapped writer
        // must fail the fence. No create, lock-file, ACL change or new ObjectId.
        let directory=std::fs::OpenOptions::new().read(true).share_mode(1).custom_flags(0x02200000)
            .open(root).map_err(|_|Failure::UnsupportedDurability)?;
        let mut files=Vec::new();let mut digests=Vec::new();
        // Respect the existing writer/maintenance lease before fencing DB files.
        // Read the existing zero-byte lock only; never create/renew a lease.
        for (name,limit) in [("vc04_adapter.lock",0u64),("vc04_snapshot_v2.sqlite3",268435456u64),
            ("vc04_snapshot_v2.sqlite3-wal",268435456),("vc04_snapshot_v2.sqlite3-shm",16777216)] {
            let path=root.join(name);super::super::reject_reparse(&path)?;
            let mut file=std::fs::OpenOptions::new().read(true).share_mode(1).custom_flags(0x00200000)
                .open(&path).map_err(|_|Failure::UnsupportedDurability)?;
            let meta=file.metadata().map_err(|_|Failure::UnsupportedDurability)?;
            if !meta.is_file()||meta.len()>limit{return Err(Failure::UnsupportedDurability);}
            digests.push(Self::digest(&mut file)?);files.push(file);
        }
        Ok(Self{_root:directory,files,digests})
    }
    fn unchanged(&mut self)->Result<(),Failure>{
        for (file,expected) in self.files.iter_mut().zip(&self.digests){
            if Self::digest(file)?!=*expected{return Err(Failure::CustodyConflict);}
        }Ok(())
    }
}
// Same schema predicates as the existing admitted adapter, without its
// writable connection/durability probe. Unknown format is not auto-migrated.
fn readonly_source_schema(db:&rusqlite::Connection)->Result<(),Failure>{
    super::super::validate_connection_schema(db,super::super::NativeSchemaProfile::SourceRevision)
}

fn readonly_unused_has_no_transition(grant:&Grant2)->Result<(),Failure>{
    let key=WindowsRootGrantStoreV1::key(Namespace::SourceTransition,&grant.base.nonce,false,false)?;
    let name=utf16(Namespace::SourceTransition.value_name());let mut kind=0;let mut length=0;
    let status=unsafe{RegQueryValueExW(key.0,name.as_ptr(),null_mut(),&mut kind,null_mut(),&mut length)};
    if status==2{Ok(())}else{Err(Failure::RecoveryRequired)}
}
impl super::super::SqliteAdapterV1 {
    pub(crate) fn observe_source_readonly(root:&std::path::Path,locator:&str,project:&[u8],
        expected_revision:&[u8],expected_integrity:&[u8;32],expected_artifact:&str)->Result<String,Failure>{
        if project.len()!=32||expected_revision.is_empty()||expected_revision.len()>4096||locator.len()!=64 {
            return Err(Failure::InvalidInput);
        }
        let nonce:[u8;32]=hex::decode(locator).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?;
        // Read existing protected data only. No host-store key is created here.
        let grant=WindowsRootGrantStoreV1::read_source_grant(&nonce)?;
        let mut files=ReadonlySourceFiles::open(root)?;
        let path=root.join("vc04_snapshot_v2.sqlite3");
        let pinned=PinnedNativeRootV1::capture(&path,false)?;
        let db=rusqlite::Connection::open_with_flags(&path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY|
            rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX).map_err(|_|Failure::UnsupportedDurability)?;
        // No immutable URI, writable adapter connect, capability or checkpoint.
        super::super::bound_sqlite_records(&db)?;
        db.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON; PRAGMA read_uncommitted=OFF; BEGIN").map_err(|_|Failure::UnsupportedDurability)?;
        let version:i64=db.query_row("PRAGMA user_version",[],|r|r.get(0)).map_err(|_|Failure::UnsupportedDurability)?;
        let mode:String=db.query_row("PRAGMA journal_mode",[],|r|r.get(0)).map_err(|_|Failure::UnsupportedDurability)?;
        if version!=2||mode!="wal"{return Err(Failure::UnsupportedDurability);}
        readonly_source_schema(&db)?;
        verify_source_observation_root(&grant,&pinned,&db,project)?;
        let head=authoring_head_on(&db,project)?;
        if head.revision!=expected_revision||head.integrity!=*expected_integrity{return Err(Failure::StaleRevision);}
        let (source,attempt,transition)=match &grant.state {
            FirstUse::Unused=>{
                WindowsRootGrantStoreV1::require_no_source_attempt(&grant)?;
                readonly_unused_has_no_transition(&grant)?;
                if head.kind!=0||head.key!=head.integrity{return Err(Failure::RecoveryRequired);}
                let count=|table:&str|->Result<i64,Failure>{db.query_row(
                    &format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 65537)"),[],|r|r.get(0))
                    .map_err(super::super::database_error)};
                let n=count("vc04_project_revisions")?;
                let non_source:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_project_revisions WHERE payload_kind!=0 LIMIT 1)",[],|r|r.get(0))
                    .map_err(super::super::database_error)?;
                if !(1..=65536).contains(&n)||non_source!=0||count("vc04_prepared_payloads")?!=n
                    ||count("vc04_project_heads")?!=1||count("vc04_recovery_custody_v1")?!=0 {
                    return Err(Failure::CustodyConflict);
                }
                let chain=super::super::source_chain::read_retained_source_ancestry(&db,project,&head.revision)?;
                if chain.initial.revision()!=grant.initial.revision||*chain.initial.integrity()!=grant.initial.integrity
                    ||*chain.initial.pesi()!=grant.base.registration_pesi||chain.latest.integrity()!=&head.integrity {
                    return Err(Failure::CustodyConflict);
                }
                (chain.latest,None,None)
            },
            FirstUse::Started(first)=>{
                let attempt=WindowsRootGrantStoreV1::read_attempt(&grant.base)?;
                if attempt.mode!=AttemptMode::RecordWitness{return Err(Failure::RecoveryRequired);}
                let transition=WindowsRootGrantStoreV1::read_transition_record(&grant)?;
                if transition.as_ref().is_some_and(|t|!matches!(t.phase,TransitionPhase::Ready|TransitionPhase::Bound)) {
                    return Err(Failure::RecoveryRequired);
                }
                let history=super::super::source_world::check_mixed_source_world_history(&db,project,attempt.epoch,
                    &attempt.transaction,&attempt.request_hash,&attempt.token_hash,&attempt.record_hash,transition.as_ref())?;
                let source=super::super::source_chain::read(&db,project,&history.latest_source_revision)?;
                let initial=super::super::source_chain::read(&db,project,&history.initial_source_revision)?;
                if grant.initial.revision!=history.initial_source_revision||grant.initial.integrity!=history.initial_source_integrity
                    ||grant.base.registration_pesi!=*initial.pesi()||first.revision!=history.first_runtime_source_revision
                    ||first.integrity!=history.first_runtime_source_integrity{return Err(Failure::CustodyConflict);}
                (source,Some(attempt),transition)
            },
        };
        // Common source restoration validates actual artifact/manifest/dependency
        // bytes. A sidecar name, digest or old success JSON is never admission.
        let artifact=source.owned().project().save().map_err(|_|Failure::CustodyConflict)?;
        if artifact!=expected_artifact{return Err(Failure::StaleRevision);}
        verify_source_observation_root(&grant,&pinned,&db,project)?;
        match attempt {
            Some(ref old)=>{
                if WindowsRootGrantStoreV1::read_attempt(&grant.base)?!=*old
                    ||WindowsRootGrantStoreV1::read_transition_record(&grant)?!=transition{return Err(Failure::CustodyConflict);}
            },
            None=>{WindowsRootGrantStoreV1::require_no_source_attempt(&grant)?;readonly_unused_has_no_transition(&grant)?;},
        }
        if authoring_head_on(&db,project)?!=head{return Err(Failure::StaleRevision);}
        db.execute_batch("COMMIT").map_err(|_|Failure::RereadFailed)?;
        db.close().map_err(|_|Failure::RereadFailed)?;
        files.unchanged()?;
        // Plain private read-time facts only; no Deserialize or conversion to
        // source admission, lease, first-use Start, Committed or replay handle.
        Ok(serde_json::json!({"kind":"native_source_readonly_observation","project_hex":hex::encode(project),
            "head_revision_hex":hex::encode(&head.revision),"head_integrity_hex":hex::encode(head.integrity),
            "head_kind":head.kind,"source_revision_hex":hex::encode(source.revision()),
            "source_integrity_hex":hex::encode(source.integrity()),"authored_project_json_utf8":artifact,
            "results":"NOT_PRODUCED","execution_authority":false,"write_authority":false,
            "durable_commit_receipt":false,"original_files_unchanged":true}).to_string())
    }
}

#[cfg(test)]
mod readonly_observation_tests {
    use super::*;
    use sha2::{Digest,Sha256};
    fn fixture()->(std::path::PathBuf,String,crate::runtime_surface::NativeSourceRevisionReadback){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root=std::env::temp_dir().join(format!("ddn-v9-readonly-test-{}",std::process::id()))
            .join(hex::encode(rand_bytes()));std::fs::create_dir_all(&root).unwrap();
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.".into()}]},String::new()).unwrap().save().unwrap();
        let saved=crate::runtime_surface::save_initial_authored_source_native(&artifact,&[1],&root).unwrap();(root,artifact,saved)
    }
    fn rand_bytes()->[u8;16]{let mut n=[0;16];getrandom::fill(&mut n).unwrap();n}
    fn inventory(root:&std::path::Path)->Vec<(String,[u8;32])>{
        let mut rows=std::fs::read_dir(root).unwrap().map(|p|{let p=p.unwrap().path();(p.file_name().unwrap().to_string_lossy().into_owned(),Sha256::digest(std::fs::read(&p).unwrap()).into())}).collect::<Vec<_>>();rows.sort();rows
    }
    fn observe(root:&std::path::Path,a:&str,s:&crate::runtime_surface::NativeSourceRevisionReadback)->Result<String,Failure>{
        super::super::super::SqliteAdapterV1::observe_source_readonly(root,s.grant_locator.as_deref().unwrap(),&s.project_identity,&s.revision_identity,&s.revision_integrity,a)
    }
    #[test]
    fn readonly_observation_native_roundtrip_facts_never_become_admission(){
        let (root,a,s)=fixture();let before=inventory(&root);let result=observe(&root,&a,&s).unwrap();let j:serde_json::Value=serde_json::from_str(&result).unwrap();
        assert_eq!(j["authored_project_json_utf8"],a);assert_eq!(j["results"],"NOT_PRODUCED");
        for key in ["execution_authority","write_authority","durable_commit_receipt"]{assert_eq!(j[key],false);}
        assert!(crate::authored_project::operate("run",&result).is_err());assert_eq!(before,inventory(&root));
        println!("V9 readonly SHA inventory {:?}; after {:?}; observation rejected as executable artifact",before,inventory(&root));
    }
    #[test]
    fn readonly_observation_started_world_checks_actual_mixed_history(){
        let (root,a,s)=fixture();let locator=s.grant_locator.as_deref().unwrap();
        let prepared=crate::runtime_surface::begin_first_authored_source_native(&root,&s.project_identity,locator,
            &s.revision_identity,&s.revision_integrity).unwrap().execute(1,0,None,None).unwrap().prepare(&[2]).unwrap();
        let root_copy=root.clone();
        prepared.stage_restore_cache(move|artifact,digest|{
            std::fs::write(root_copy.join(format!("test-world-{}.json",hex::encode(digest))),artifact)
                .map_err(|e|format!("{e}"))
        }).unwrap();let retained=prepared.write_prepared().unwrap().commit().unwrap();drop(retained);
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (revision,integrity):(Vec<u8>,Vec<u8>)=db.query_row("SELECT history_revision_identity,revision_integrity_sha256 FROM vc04_project_heads",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();drop(db);
        let integrity:[u8;32]=integrity.try_into().unwrap();let before=inventory(&root);
        let result=super::super::super::SqliteAdapterV1::observe_source_readonly(&root,locator,&s.project_identity,&revision,&integrity,&a).unwrap();
        let j:serde_json::Value=serde_json::from_str(&result).unwrap();assert_eq!(j["head_kind"],1);assert_eq!(j["results"],"NOT_PRODUCED");assert_eq!(before,inventory(&root));
        println!("V9 started mixed-history SHA inventory {:?}; after {:?}",before,inventory(&root));
    }
    #[test]
    fn readonly_observation_stale_and_foreign_grant_refuse_without_changes(){
        let (root,a,s)=fixture();let (foreign,_,other)=fixture();let before=inventory(&root);let foreign_before=inventory(&foreign);
        let stale=super::super::super::SqliteAdapterV1::observe_source_readonly(&root,s.grant_locator.as_deref().unwrap(),&s.project_identity,&[99],&s.revision_integrity,&a);
        assert!(matches!(stale,Err(Failure::StaleRevision)),"{stale:?}");
        let wrong=super::super::super::SqliteAdapterV1::observe_source_readonly(&root,other.grant_locator.as_deref().unwrap(),&s.project_identity,&s.revision_identity,&s.revision_integrity,&a);
        assert!(matches!(wrong,Err(Failure::CustodyConflict)),"{wrong:?}");assert_eq!(before,inventory(&root));assert_eq!(foreign_before,inventory(&foreign));
    }
    #[test]
    fn readonly_observation_existing_or_concurrent_writer_is_unsupported(){
        use std::os::windows::fs::OpenOptionsExt;
        let (root,a,s)=fixture();let before=inventory(&root);
        let writer=std::fs::OpenOptions::new().write(true).share_mode(3).open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
        assert!(matches!(observe(&root,&a,&s),Err(Failure::UnsupportedDurability)));drop(writer);
        let owner_lease=std::fs::OpenOptions::new().read(true).write(true).share_mode(0).open(root.join("vc04_adapter.lock")).unwrap();
        assert!(matches!(observe(&root,&a,&s),Err(Failure::UnsupportedDurability)));drop(owner_lease);
        let guard=ReadonlySourceFiles::open(&root).unwrap();
        assert!(std::fs::OpenOptions::new().read(true).write(true).share_mode(0).open(root.join("vc04_adapter.lock")).is_err());
        assert!(std::fs::OpenOptions::new().write(true).share_mode(3).open(root.join("vc04_snapshot_v2.sqlite3-wal")).is_err());drop(guard);
        assert_eq!(before,inventory(&root));
    }
    #[test]
    fn readonly_observation_corrupt_source_is_not_repaired(){
        let (root,a,s)=fixture();let db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
        db.execute("UPDATE vc04_prepared_payloads SET project_payload_bytes=zeroblob(length(project_payload_bytes)) WHERE payload_kind=0",[]).unwrap();drop(db);
        let before=inventory(&root);assert!(observe(&root,&a,&s).is_err());assert_eq!(before,inventory(&root));
        let (root,a,s)=fixture();let nonce:[u8;32]=hex::decode(s.grant_locator.as_deref().unwrap()).unwrap().try_into().unwrap();
        let invalid=b"invalid-test-only-transition";
        WindowsRootGrantStoreV1::write_plaintext(Namespace::SourceTransition,&nonce,invalid,false).unwrap();
        let before=inventory(&root);assert!(matches!(observe(&root,&a,&s),Err(Failure::RecoveryRequired)));
        assert_eq!(WindowsRootGrantStoreV1::read_plaintext(Namespace::SourceTransition,&nonce).unwrap(),invalid);assert_eq!(before,inventory(&root));
        let (root,a,s)=fixture();let db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
        db.execute_batch("CREATE VIEW unexpected_schema AS SELECT 1").unwrap();drop(db);let before=inventory(&root);
        assert!(matches!(observe(&root,&a,&s),Err(Failure::UnsupportedDurability)));assert_eq!(before,inventory(&root));
        for ddl in ["CREATE INDEX unsupported_index ON vc04_project_heads(project_identity)",
            "CREATE TRIGGER unsupported_trigger AFTER UPDATE ON vc04_project_heads BEGIN SELECT 1; END"] {
            let (root,a,s)=fixture();let db=rusqlite::Connection::open(root.join("vc04_snapshot_v2.sqlite3")).unwrap();
            db.execute_batch(ddl).unwrap();drop(db);let before=inventory(&root);
            assert!(matches!(observe(&root,&a,&s),Err(Failure::UnsupportedDurability)));assert_eq!(before,inventory(&root));
        }
    }
    #[test]
    fn readonly_observation_missing_sidecars_are_not_created(){
        let (root,a,s)=fixture();let absent=root.join("not-a-store");let result=observe(&absent,&a,&s);
        assert!(result.is_err());assert!(!absent.exists());
        let remote=std::path::Path::new(r"\\not-authorized\share\original");
        assert!(matches!(observe(remote,&a,&s),Err(Failure::UnsupportedDurability)));
    }
}
