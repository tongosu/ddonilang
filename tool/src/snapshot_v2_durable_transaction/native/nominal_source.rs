//! New user_version=5 profile. Old profiles cannot open or migrate it.
use super::{SqliteAdapterV1,NativeSchemaProfile,Failure,database_error,sync_directory};
use crate::unit_wire::project_runtime_attachment::nominal_source::{NominalSource,Anchor};
use rusqlite::{params,OptionalExtension};
use sha2::{Digest,Sha256};
const PAYLOAD:&str="CREATE TABLE vc04_nominal_payloads(digest BLOB PRIMARY KEY CHECK(length(digest)=32), artifact BLOB NOT NULL CHECK(length(artifact)>0 AND length(artifact)<=8000000)) WITHOUT ROWID";
const REVISIONS:&str="CREATE TABLE vc04_nominal_revisions(project BLOB NOT NULL CHECK(length(project)=32), revision BLOB NOT NULL CHECK(length(revision)=16), parent BLOB NOT NULL CHECK(length(parent) IN (0,16)), integrity BLOB NOT NULL CHECK(length(integrity)=32), digest BLOB NOT NULL REFERENCES vc04_nominal_payloads(digest), PRIMARY KEY(project,revision)) WITHOUT ROWID";
const HEADS:&str="CREATE TABLE vc04_nominal_heads(project BLOB PRIMARY KEY CHECK(length(project)=32), revision BLOB NOT NULL CHECK(length(revision)=16), integrity BLOB NOT NULL CHECK(length(integrity)=32), FOREIGN KEY(project,revision) REFERENCES vc04_nominal_revisions(project,revision)) WITHOUT ROWID";
pub(super) fn tables()->[(&'static str,&'static str);3]{[("vc04_nominal_heads",HEADS),("vc04_nominal_payloads",PAYLOAD),("vc04_nominal_revisions",REVISIONS)]}
#[cfg(test)]
std::thread_local!{static FAULT:std::cell::Cell<u8>=const{std::cell::Cell::new(0)};}
impl SqliteAdapterV1 {
    #[cfg(windows)]
    fn nominal_candidate_path(&self,revision:&[u8;16])->Result<std::path::PathBuf,String>{
        Ok(self.path.parent().ok_or("E_NOMINAL_SOURCE_ROOT")?.join(format!("nominal-source-candidate-{}.json",hex::encode(revision))))
    }
    #[cfg(windows)]
    fn read_nominal_candidate(&self,anchor:&Anchor)->Result<NominalSource,String>{
        use std::io::Read;
        let path=self.nominal_candidate_path(&anchor.revision)?;
        let meta=std::fs::symlink_metadata(&path).map_err(|_|"E_NOMINAL_SOURCE_PENDING_CANDIDATE_REQUIRED")?;
        if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>8_000_000{return Err("E_NOMINAL_SOURCE_PENDING_CANDIDATE_FIELDS".into());}
        let mut bytes=Vec::new();std::fs::File::open(path).map_err(|_|"E_NOMINAL_SOURCE_PENDING_CANDIDATE_READ")?.take(8_000_001).read_to_end(&mut bytes).map_err(|_|"E_NOMINAL_SOURCE_PENDING_CANDIDATE_READ")?;
        if bytes.len()>8_000_000{return Err("E_NOMINAL_SOURCE_PENDING_CANDIDATE_FIELDS".into());}
        let candidate=NominalSource::decode(std::str::from_utf8(&bytes).map_err(|_|"E_NOMINAL_SOURCE_PENDING_CANDIDATE_UTF8")?)?;
        if candidate.anchor()?!=*anchor{return Err("E_NOMINAL_SOURCE_PENDING_CANDIDATE_CONTEXT".into());}Ok(candidate)
    }
    #[cfg(windows)]
    fn write_nominal_candidate(&self,candidate:&NominalSource)->Result<(),String>{
        use std::io::Write;
        let anchor=candidate.anchor()?;let path=self.nominal_candidate_path(&anchor.revision)?;
        let encoded=candidate.encode()?;
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path){
            Ok(mut file)=>{file.write_all(encoded.as_bytes()).map_err(|_|"E_NOMINAL_SOURCE_CANDIDATE_WRITE")?;file.sync_all().map_err(|_|"E_NOMINAL_SOURCE_CANDIDATE_SYNC")?;},
            Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{if self.read_nominal_candidate(&anchor)?.encode()!=Ok(encoded){return Err("E_NOMINAL_SOURCE_CANDIDATE_CONFLICT".into());}},
            Err(_)=>return Err("E_NOMINAL_SOURCE_CANDIDATE_WRITE".into()),
        }
        sync_directory(self.path.parent().ok_or("E_NOMINAL_SOURCE_ROOT")?).map_err(|e|format!("E_NOMINAL_SOURCE_CANDIDATE_SYNC: {e:?}"))
    }
    pub(crate) fn provision_nominal_source(root:&std::path::Path)->Result<Self,Failure>{Self::provision_profile(root,NativeSchemaProfile::NominalSource)}
    pub(crate) fn open_nominal_source(root:&std::path::Path)->Result<Self,Failure>{Self::open_admitted_profile(root,NativeSchemaProfile::NominalSource)}
    fn nominal_database_current(&self,project:&[u8;32])->Result<Option<NominalSource>,Failure>{
        if self.schema_profile!=NativeSchemaProfile::NominalSource{return Err(Failure::UnsupportedDurability);}
        let c=self.connect()?;
        let row:Option<(Vec<u8>,Vec<u8>)>=c.query_row("SELECT revision,integrity FROM vc04_nominal_heads WHERE project=?1",params![project.as_slice()],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(database_error)?;
        let Some((mut rev,integrity))=row else{return Ok(None)};let mut result=None;let mut seen=std::collections::BTreeSet::new();
        loop {
            if seen.len()>=4096||!seen.insert(rev.clone()){return Err(Failure::CustodyConflict);}
            let (parent,anchor,digest,raw):(Vec<u8>,Vec<u8>,Vec<u8>,Vec<u8>)=c.query_row("SELECT r.parent,r.integrity,r.digest,p.artifact FROM vc04_nominal_revisions r JOIN vc04_nominal_payloads p ON p.digest=r.digest WHERE r.project=?1 AND r.revision=?2",params![project.as_slice(),&rev],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(database_error)?;
            if raw.len()>8_000_000||Sha256::digest(&raw).as_slice()!=digest.as_slice(){return Err(Failure::CustodyConflict);}
            let source=NominalSource::decode(std::str::from_utf8(&raw).map_err(|_|Failure::CustodyConflict)?).map_err(|_|Failure::CustodyConflict)?;
            let actual=source.anchor().map_err(|_|Failure::CustodyConflict)?;
            if source.project().map_err(|_|Failure::CustodyConflict)?!=*project||actual.revision.as_slice()!=rev
                ||actual.integrity.as_slice()!=anchor||hex::decode(&source.parent_hex).map_err(|_|Failure::CustodyConflict)?!=parent{return Err(Failure::CustodyConflict);}
            if result.is_none(){if actual.integrity.as_slice()!=integrity{return Err(Failure::CustodyConflict);}result=Some(source);}
            if parent.is_empty(){break;}rev=parent;
        }
        Ok(result)
    }
    #[cfg(windows)]
    pub(crate) fn read_nominal_source(&mut self,project:&[u8;32])->Result<NominalSource,String>{
        let current=self.nominal_database_current(project).map_err(|e|format!("E_NOMINAL_SOURCE_READ: {e:?}"))?;
        self.resolve_nominal_source_grant(project,current.as_ref().map(NominalSource::anchor).transpose()?.as_ref())
            .map_err(|e|format!("E_NOMINAL_SOURCE_GRANT: {e:?}"))?;
        current.ok_or_else(||"E_NOMINAL_SOURCE_UNCONFIRMED: 저장된 원문이 없습니다. 같은 원문의 명시 저장으로 확인하세요".into())
    }
    #[cfg(windows)]
    pub(crate) fn save_nominal_source(&mut self,raw:&str,previous:Option<&str>,expected:Option<&Anchor>,next:[u8;16],fresh:bool)->Result<NominalSource,String>{
        let p=crate::unit_wire::project_runtime_attachment::nominal_source::checked_project(raw)?.ok_or("E_NOMINAL_SOURCE_REQUIRED")?;
        let project:[u8;32]=hex::decode(p.authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_NOMINAL_SOURCE_PROJECT")?.try_into().map_err(|_|"E_NOMINAL_SOURCE_PROJECT")?;
        let current=self.nominal_database_current(&project).map_err(|e|format!("E_NOMINAL_SOURCE_READ: {e:?}"))?;
        if current.as_ref().is_some_and(|old|old.authored_project_json_utf8!=raw){
            let count:i64=self.connect().map_err(|_|"E_NOMINAL_SOURCE_OPEN")?.query_row("SELECT count(*) FROM vc04_nominal_revisions WHERE project=?1",params![project.as_slice()],|r|r.get(0)).map_err(|_|"E_NOMINAL_SOURCE_READ")?;
            if count>=4096{return Err("E_NOMINAL_SOURCE_REVISION_LIMIT".into());}
        }
        // Validate the actual selected protected parent before invoking the issuer.
        let candidate=if fresh {
            if current.is_some(){return Err("E_NOMINAL_SOURCE_NOT_EMPTY".into());}
            let candidate=NominalSource::prepare(raw,None,"",&hex::encode(next))?;
            self.write_nominal_candidate(&candidate)?;
            self.register_nominal_source_grant(&project,&candidate.anchor()?).map_err(|e|format!("E_NOMINAL_SOURCE_GRANT_REGISTER: {e:?}"))?;
            candidate
        } else {
            let pending=self.resolve_nominal_source_grant(&project,current.as_ref().map(NominalSource::anchor).transpose()?.as_ref())
                .map_err(|e|format!("E_NOMINAL_SOURCE_GRANT: {e:?}"))?;
            if let Some(old)=&current {
                if old.authored_project_json_utf8==raw{return Ok(old.clone());}
                if previous!=Some(old.authored_project_json_utf8.as_str())||expected!=Some(&old.anchor()?)||old.profile!=p.source_profile{return Err("E_NOMINAL_SOURCE_STALE_PARENT: 저장본이 바뀌었습니다. 현재 편집은 보존했습니다".into());}
                let candidate=NominalSource::prepare(raw,Some(old),&old.revision_hex,&hex::encode(next))?;
                self.write_nominal_candidate(&candidate)?;
                self.begin_nominal_source_update(&project,Some(&old.anchor()?),&candidate.anchor()?).map_err(|e|format!("E_NOMINAL_SOURCE_GRANT_PENDING: {e:?}"))?;
                candidate
            } else {
                // Registration IDs are nondeterministic. A first-save COMMIT gap
                // must use the exact frozen candidate authenticated by Pending,
                // never call the issuer again and hope for an equivalent result.
                let anchor=pending.ok_or("E_NOMINAL_SOURCE_UNCONFIRMED")?;
                let candidate=self.read_nominal_candidate(&anchor)?;
                if candidate.project()?!=project||candidate.authored_project_json_utf8!=raw||!candidate.parent_hex.is_empty(){return Err("E_NOMINAL_SOURCE_UNCONFIRMED_DIFFERENT_SOURCE".into());}
                candidate.fresh_context()?;
                self.begin_nominal_source_update(&project,None,&anchor).map_err(|e|format!("E_NOMINAL_SOURCE_GRANT_PENDING: {e:?}"))?;
                candidate
            }
        };
        let encoded=candidate.encode()?;let digest:[u8;32]=Sha256::digest(encoded.as_bytes()).into();let anchor=candidate.anchor()?;
        let c=self.connect().map_err(|e|format!("E_NOMINAL_SOURCE_OPEN: {e:?}"))?;
        let save=(||->Result<(),Failure>{
            c.execute_batch("BEGIN IMMEDIATE").map_err(database_error)?;
            let observed:Option<Vec<u8>>=c.query_row("SELECT revision FROM vc04_nominal_heads WHERE project=?1",params![project.as_slice()],|r|r.get(0)).optional().map_err(database_error)?;
            if observed!=current.as_ref().map(|x|hex::decode(&x.revision_hex).unwrap()){return Err(Failure::StaleRevision);}
            c.execute("INSERT INTO vc04_nominal_payloads(digest,artifact) VALUES(?1,?2)",params![digest.as_slice(),encoded.as_bytes()]).map_err(database_error)?;
            c.execute("INSERT INTO vc04_nominal_revisions(project,revision,parent,integrity,digest) VALUES(?1,?2,?3,?4,?5)",params![project.as_slice(),anchor.revision.as_slice(),hex::decode(&candidate.parent_hex).map_err(|_|Failure::InvalidInput)?,anchor.integrity.as_slice(),digest.as_slice()]).map_err(database_error)?;
            c.execute("INSERT INTO vc04_nominal_heads(project,revision,integrity) VALUES(?1,?2,?3) ON CONFLICT(project) DO UPDATE SET revision=excluded.revision,integrity=excluded.integrity",params![project.as_slice(),anchor.revision.as_slice(),anchor.integrity.as_slice()]).map_err(database_error)?;
            #[cfg(test)] if FAULT.with(|x|if x.get()==1{x.set(0);true}else{false}){return Err(Failure::WriteFailed);}
            c.execute_batch("COMMIT").map_err(database_error)?;Ok(())
        })();drop(c);save.map_err(|e|format!("E_NOMINAL_SOURCE_SAVE_UNCONFIRMED: {e:?}"))?;
        #[cfg(test)] if FAULT.with(|x|if x.get()==2{x.set(0);true}else{false}){return Err("E_NOMINAL_SOURCE_SAVE_UNCONFIRMED: injected after COMMIT".into());}
        self.capability().map_err(|e|format!("E_NOMINAL_SOURCE_SYNC_UNCONFIRMED: {e:?}"))?;
        sync_directory(self.path.parent().ok_or("E_NOMINAL_SOURCE_ROOT")?).map_err(|e|format!("E_NOMINAL_SOURCE_SYNC_UNCONFIRMED: {e:?}"))?;
        let read=self.read_nominal_source(&project)?;
        if read.encode()?!=encoded{return Err("E_NOMINAL_SOURCE_READBACK".into());}Ok(read)
    }
}


#[cfg(all(test,windows))]mod tests{
 use super::*;use crate::unit_wire::project_runtime_attachment::nominal_source::tests::{artifact,edited,SOURCE};
 fn root()->std::path::PathBuf{let mut id=[0;16];getrandom::fill(&mut id).unwrap();let root=std::env::temp_dir().join(format!("ddn-nominal-source-{}",hex::encode(id)));std::fs::create_dir(&root).unwrap();root}
 #[test]fn nominal_source_native_cold_cas_keeps_registration_and_refuses_old_profile(){
  let root=root();let raw=artifact(SOURCE);let mut db=SqliteAdapterV1::provision_nominal_source(&root).unwrap();let a=db.save_nominal_source(&raw,None,None,[1;16],true).unwrap();let id=a.project().unwrap();let anchor=a.anchor().unwrap();drop(db);
  assert!(SqliteAdapterV1::open_code_source(&root).is_err());let mut db=SqliteAdapterV1::open_nominal_source(&root).unwrap();assert_eq!(db.read_nominal_source(&id).unwrap().encode().unwrap(),a.encode().unwrap());
  let changed=edited(&raw,&SOURCE.replace("접수 <- 거짓으로 구매.","접수 <- 참으로 구매."));let b=db.save_nominal_source(&changed,Some(&raw),Some(&anchor),[2;16],false).unwrap();let bytes=b.encode().unwrap();drop(db);
  let mut db=SqliteAdapterV1::open_nominal_source(&root).unwrap();let cold=db.read_nominal_source(&id).unwrap();assert_eq!(cold.encode().unwrap(),bytes);cold.fresh_context().unwrap();
  assert!(db.save_nominal_source(&raw,Some(&raw),Some(&anchor),[3;16],false).is_err());assert_eq!(db.read_nominal_source(&id).unwrap().encode().unwrap(),bytes);
 }
 #[test]fn nominal_source_first_commit_gap_reuses_frozen_ids_and_existing_head_survives_failure(){
  let root=root();let raw=artifact(SOURCE);let mut db=SqliteAdapterV1::provision_nominal_source(&root).unwrap();FAULT.with(|f|f.set(1));assert!(db.save_nominal_source(&raw,None,None,[4;16],true).is_err());drop(db);
  let mut db=SqliteAdapterV1::open_nominal_source(&root).unwrap();let a=db.save_nominal_source(&raw,None,None,[5;16],false).unwrap();assert_eq!(a.anchor().unwrap().revision,[4;16]);let id=a.project().unwrap();let bytes=a.encode().unwrap();
  let changed=edited(&raw,&SOURCE.replace("접수 <- 거짓으로 구매.","접수 <- 참으로 구매."));FAULT.with(|f|f.set(1));assert!(db.save_nominal_source(&changed,Some(&raw),Some(&a.anchor().unwrap()),[6;16],false).is_err());drop(db);
  let mut db=SqliteAdapterV1::open_nominal_source(&root).unwrap();assert_eq!(db.read_nominal_source(&id).unwrap().encode().unwrap(),bytes);
  FAULT.with(|f|f.set(2));assert!(db.save_nominal_source(&changed,Some(&raw),Some(&a.anchor().unwrap()),[7;16],false).is_err());drop(db);
  let mut db=SqliteAdapterV1::open_nominal_source(&root).unwrap();let b=db.read_nominal_source(&id).unwrap();assert_eq!(b.anchor().unwrap().revision,[7;16]);assert_eq!(b.authored_project_json_utf8,changed);b.fresh_context().unwrap();
 }
}
