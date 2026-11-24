//! New user_version=4 profile. Old profiles cannot open or migrate it.
use super::{SqliteAdapterV1,NativeSchemaProfile,Failure,database_error,sync_directory};
use crate::unit_wire::project_runtime_attachment::code_source::{CodeSource,Anchor};
use rusqlite::{params,OptionalExtension};
use sha2::{Digest,Sha256};
const PAYLOAD:&str="CREATE TABLE vc04_code_payloads(digest BLOB PRIMARY KEY CHECK(length(digest)=32), artifact BLOB NOT NULL CHECK(length(artifact)>0 AND length(artifact)<=2000000)) WITHOUT ROWID";
const REVISIONS:&str="CREATE TABLE vc04_code_revisions(project BLOB NOT NULL CHECK(length(project)=32), revision BLOB NOT NULL CHECK(length(revision)=16), parent BLOB NOT NULL CHECK(length(parent) IN (0,16)), integrity BLOB NOT NULL CHECK(length(integrity)=32), digest BLOB NOT NULL REFERENCES vc04_code_payloads(digest), PRIMARY KEY(project,revision)) WITHOUT ROWID";
const HEADS:&str="CREATE TABLE vc04_code_heads(project BLOB PRIMARY KEY CHECK(length(project)=32), revision BLOB NOT NULL CHECK(length(revision)=16), integrity BLOB NOT NULL CHECK(length(integrity)=32), FOREIGN KEY(project,revision) REFERENCES vc04_code_revisions(project,revision)) WITHOUT ROWID";
pub(super) fn tables()->[(&'static str,&'static str);3]{[("vc04_code_heads",HEADS),("vc04_code_payloads",PAYLOAD),("vc04_code_revisions",REVISIONS)]}
#[cfg(test)]
std::thread_local!{static FAULT:std::cell::Cell<u8>=const{std::cell::Cell::new(0)};}
impl SqliteAdapterV1 {
    pub(crate) fn provision_code_source(root:&std::path::Path)->Result<Self,Failure>{Self::provision_profile(root,NativeSchemaProfile::CodeSource)}
    pub(crate) fn open_code_source(root:&std::path::Path)->Result<Self,Failure>{Self::open_admitted_profile(root,NativeSchemaProfile::CodeSource)}
    fn code_database_current(&self,project:&[u8;32])->Result<Option<CodeSource>,Failure>{
        if self.schema_profile!=NativeSchemaProfile::CodeSource{return Err(Failure::UnsupportedDurability);}
        let c=self.connect()?;
        let row:Option<(Vec<u8>,Vec<u8>)>=c.query_row("SELECT revision,integrity FROM vc04_code_heads WHERE project=?1",params![project.as_slice()],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(database_error)?;
        let Some((mut rev,integrity))=row else{return Ok(None)};let mut result=None;let mut seen=std::collections::BTreeSet::new();
        loop {
            if seen.len()>=4096||!seen.insert(rev.clone()){return Err(Failure::CustodyConflict);}
            let (parent,anchor,digest,raw):(Vec<u8>,Vec<u8>,Vec<u8>,Vec<u8>)=c.query_row("SELECT r.parent,r.integrity,r.digest,p.artifact FROM vc04_code_revisions r JOIN vc04_code_payloads p ON p.digest=r.digest WHERE r.project=?1 AND r.revision=?2",params![project.as_slice(),&rev],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(database_error)?;
            if raw.len()>2_000_000||Sha256::digest(&raw).as_slice()!=digest.as_slice(){return Err(Failure::CustodyConflict);}
            let source=CodeSource::decode(std::str::from_utf8(&raw).map_err(|_|Failure::CustodyConflict)?).map_err(|_|Failure::CustodyConflict)?;
            let actual=source.anchor().map_err(|_|Failure::CustodyConflict)?;
            if source.project().map_err(|_|Failure::CustodyConflict)?!=*project||actual.revision.as_slice()!=rev
                ||actual.integrity.as_slice()!=anchor||hex::decode(&source.parent_hex).map_err(|_|Failure::CustodyConflict)?!=parent{return Err(Failure::CustodyConflict);}
            if result.is_none(){if actual.integrity.as_slice()!=integrity{return Err(Failure::CustodyConflict);}result=Some(source);}
            if parent.is_empty(){break;}rev=parent;
        }
        Ok(result)
    }
    #[cfg(windows)]
    pub(crate) fn read_code_source(&mut self,project:&[u8;32])->Result<CodeSource,String>{
        let current=self.code_database_current(project).map_err(|e|format!("E_CODE_SOURCE_READ: {e:?}"))?;
        self.resolve_code_source_grant(project,current.as_ref().map(CodeSource::anchor).transpose()?.as_ref())
            .map_err(|e|format!("E_CODE_SOURCE_GRANT: {e:?}"))?;
        current.ok_or_else(||"E_CODE_SOURCE_UNCONFIRMED: 저장된 원문이 없습니다. 같은 원문의 명시 저장으로 확인하세요".into())
    }
    #[cfg(windows)]
    pub(crate) fn save_code_source(&mut self,raw:&str,previous:Option<&str>,expected:Option<&Anchor>,next:[u8;16],fresh:bool)->Result<CodeSource,String>{
        let p=crate::unit_wire::project_runtime_attachment::code_source::checked_project(raw)?.ok_or("E_CODE_SOURCE_REQUIRED")?;
        let project:[u8;32]=hex::decode(p.authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_CODE_SOURCE_PROJECT")?.try_into().map_err(|_|"E_CODE_SOURCE_PROJECT")?;
        let current=self.code_database_current(&project).map_err(|e|format!("E_CODE_SOURCE_READ: {e:?}"))?;
        if current.as_ref().is_some_and(|old|old.authored_project_json_utf8!=raw){
            let count:i64=self.connect().map_err(|_|"E_CODE_SOURCE_OPEN")?.query_row("SELECT count(*) FROM vc04_code_revisions WHERE project=?1",params![project.as_slice()],|r|r.get(0)).map_err(|_|"E_CODE_SOURCE_READ")?;
            if count>=4096{return Err("E_CODE_SOURCE_REVISION_LIMIT".into());}
        }
        let mut candidate=CodeSource::prepare(raw,current.as_ref().map(|s|s.revision_hex.as_str()).unwrap_or(""),&hex::encode(next))?;
        if fresh{
            if current.is_some(){return Err("E_CODE_SOURCE_NOT_EMPTY".into());}
            self.register_code_source_grant(&project,&candidate.anchor()?).map_err(|e|format!("E_CODE_SOURCE_GRANT_REGISTER: {e:?}"))?;
        }else{
            let pending=self.resolve_code_source_grant(&project,current.as_ref().map(CodeSource::anchor).transpose()?.as_ref())
                .map_err(|e|format!("E_CODE_SOURCE_GRANT: {e:?}"))?;
            if let Some(old)=&current{
                if old.authored_project_json_utf8==raw{return Ok(old.clone());}
                if previous!=Some(old.authored_project_json_utf8.as_str())||expected!=Some(&old.anchor()?)||old.profile!=p.source_profile{return Err("E_CODE_SOURCE_STALE_PARENT: 저장본이 바뀌었습니다. 현재 편집은 보존했습니다".into());}
            }else{
                let anchor=pending.ok_or("E_CODE_SOURCE_UNCONFIRMED")?;
                candidate=CodeSource::prepare(raw,"",&hex::encode(anchor.revision))?;
                if candidate.anchor()?!=anchor{return Err("E_CODE_SOURCE_UNCONFIRMED_DIFFERENT_SOURCE".into());}
            }
            self.begin_code_source_update(&project,current.as_ref().map(CodeSource::anchor).transpose()?.as_ref(),&candidate.anchor()?)
                .map_err(|e|format!("E_CODE_SOURCE_GRANT_PENDING: {e:?}"))?;
        }
        let encoded=candidate.encode()?;let digest:[u8;32]=Sha256::digest(encoded.as_bytes()).into();let anchor=candidate.anchor()?;
        let c=self.connect().map_err(|e|format!("E_CODE_SOURCE_OPEN: {e:?}"))?;
        let save=(||->Result<(),Failure>{
            c.execute_batch("BEGIN IMMEDIATE").map_err(database_error)?;
            let observed:Option<Vec<u8>>=c.query_row("SELECT revision FROM vc04_code_heads WHERE project=?1",params![project.as_slice()],|r|r.get(0)).optional().map_err(database_error)?;
            if observed!=current.as_ref().map(|x|hex::decode(&x.revision_hex).unwrap()){return Err(Failure::StaleRevision);}
            c.execute("INSERT INTO vc04_code_payloads(digest,artifact) VALUES(?1,?2)",params![digest.as_slice(),encoded.as_bytes()]).map_err(database_error)?;
            c.execute("INSERT INTO vc04_code_revisions(project,revision,parent,integrity,digest) VALUES(?1,?2,?3,?4,?5)",params![project.as_slice(),anchor.revision.as_slice(),hex::decode(&candidate.parent_hex).map_err(|_|Failure::InvalidInput)?,anchor.integrity.as_slice(),digest.as_slice()]).map_err(database_error)?;
            c.execute("INSERT INTO vc04_code_heads(project,revision,integrity) VALUES(?1,?2,?3) ON CONFLICT(project) DO UPDATE SET revision=excluded.revision,integrity=excluded.integrity",params![project.as_slice(),anchor.revision.as_slice(),anchor.integrity.as_slice()]).map_err(database_error)?;
            #[cfg(test)] if FAULT.with(|x|x.get())==1{return Err(Failure::WriteFailed);}
            c.execute_batch("COMMIT").map_err(database_error)?;Ok(())
        })();drop(c);save.map_err(|e|format!("E_CODE_SOURCE_SAVE_UNCONFIRMED: {e:?}"))?;
        #[cfg(test)] if FAULT.with(|x|x.get())==2{return Err("E_CODE_SOURCE_SAVE_UNCONFIRMED: injected after COMMIT".into());}
        self.capability().map_err(|e|format!("E_CODE_SOURCE_SYNC_UNCONFIRMED: {e:?}"))?;
        sync_directory(self.path.parent().ok_or("E_CODE_SOURCE_ROOT")?).map_err(|e|format!("E_CODE_SOURCE_SYNC_UNCONFIRMED: {e:?}"))?;
        let read=self.read_code_source(&project)?;
        if read.encode()?!=encoded{return Err("E_CODE_SOURCE_READBACK".into());}Ok(read)
    }
}

#[cfg(all(test,windows))]
mod tests{
    use super::*;
    use crate::unit_wire::project_runtime_attachment::code_source::tests::{artifact,edited,SOURCE};
    fn root()->std::path::PathBuf{let mut x=[0;16];getrandom::fill(&mut x).unwrap();let p=std::env::temp_dir().join(format!("ddn-code-source-{}",hex::encode(x)));std::fs::create_dir(&p).unwrap();p}
    #[test]
    fn code_source_native_cold_cas_and_protected_rollback_refusal(){
        let root=root();let a=artifact(SOURCE);let mut adapter=SqliteAdapterV1::provision_code_source(&root).unwrap();
        let first=adapter.save_code_source(&a,None,None,[1;16],true).unwrap();let project=first.project().unwrap();let anchor=first.anchor().unwrap();drop(adapter);
        assert!(SqliteAdapterV1::open_source_revision(&root).is_err());
        let mut adapter=SqliteAdapterV1::open_code_source(&root).unwrap();assert_eq!(adapter.read_code_source(&project).unwrap().encode().unwrap(),first.encode().unwrap());
        let b=edited(&a,&SOURCE.replace("3.0","2.0"));let second=adapter.save_code_source(&b,Some(&a),Some(&anchor),[2;16],false).unwrap();
        assert_eq!(first.project().unwrap(),second.project().unwrap());assert_eq!(second.parent_hex,first.revision_hex);
        let c=edited(&a,&SOURCE.replace("3.0","4.0"));assert!(adapter.save_code_source(&c,Some(&a),Some(&anchor),[3;16],false).err().unwrap().contains("STALE_PARENT"));
        assert!(adapter.read_code_source(&[9;32]).is_err());
        let c=adapter.connect().unwrap();c.execute("UPDATE vc04_code_heads SET revision=?1,integrity=?2 WHERE project=?3",params![anchor.revision.as_slice(),anchor.integrity.as_slice(),project.as_slice()]).unwrap();drop(c);
        assert!(adapter.read_code_source(&project).is_err(),"a valid earlier DB head must not replace protected committed head");
    }
    #[test]
    fn code_source_native_unconfirmed_commit_same_revision_recovery(){
        let root=root();let a=artifact(SOURCE);let mut adapter=SqliteAdapterV1::provision_code_source(&root).unwrap();
        FAULT.with(|x|x.set(2));let error=adapter.save_code_source(&a,None,None,[4;16],true).err().unwrap();FAULT.with(|x|x.set(0));assert!(error.contains("UNCONFIRMED"));drop(adapter);
        let mut adapter=SqliteAdapterV1::open_code_source(&root).unwrap();let p=crate::authored_project::open(a.as_bytes()).unwrap();let id=hex::decode(p.authored_identity.logical_dependency_id.as_str()).unwrap().try_into().unwrap();
        let recovered=adapter.read_code_source(&id).unwrap();assert_eq!(recovered.revision_hex,"04".repeat(16));assert_eq!(recovered.authored_project_json_utf8,a);
        let no_op=adapter.save_code_source(&a,None,None,[5;16],false).unwrap();assert_eq!(no_op.encode().unwrap(),recovered.encode().unwrap());
    }
    #[test]
    fn code_source_native_before_commit_exact_pending_retry(){
        let root=root();let a=artifact(SOURCE);let mut adapter=SqliteAdapterV1::provision_code_source(&root).unwrap();
        FAULT.with(|x|x.set(1));assert!(adapter.save_code_source(&a,None,None,[6;16],true).is_err());FAULT.with(|x|x.set(0));drop(adapter);
        let mut adapter=SqliteAdapterV1::open_code_source(&root).unwrap();let b=edited(&a,&SOURCE.replace("3.0","2.0"));assert!(adapter.save_code_source(&b,None,None,[7;16],false).is_err());
        let saved=adapter.save_code_source(&a,None,None,[7;16],false).unwrap();assert_eq!(saved.revision_hex,"06".repeat(16));assert_eq!(saved.authored_project_json_utf8,a);
    }
}
