// Uses the existing SID/NTFS object pins and two DPAPI layers. This disjoint
// record can authorize code source custody only, never a first world or Run.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeGrantRecord {
    nonce:[u8;32],fingerprint:[u8;32],project:[u8;32],
    committed:Option<crate::unit_wire::project_runtime_attachment::code_source::Anchor>,
    pending:Option<crate::unit_wire::project_runtime_attachment::code_source::Anchor>,
}
impl CodeGrantRecord {
    fn bytes(&self)->Result<Vec<u8>,Failure>{
        if self.nonce==[0;32]||(self.committed.is_none()&&self.pending.is_none())||self.committed==self.pending{return Err(Failure::CustodyConflict);}
        let mut out=b"DDN-CODE-SOURCE-GRANT-V1\0".to_vec();out.extend(serde_json::to_vec(self).map_err(|_|Failure::InvalidInput)?);Ok(out)
    }
    fn decode(raw:&[u8])->Result<Self,Failure>{
        let body=raw.strip_prefix(b"DDN-CODE-SOURCE-GRANT-V1\0").ok_or(Failure::CustodyConflict)?;
        let r:Self=serde_json::from_slice(body).map_err(|_|Failure::CustodyConflict)?;
        if r.bytes()?!=raw{return Err(Failure::CustodyConflict);}Ok(r)
    }
}
impl super::super::SqliteAdapterV1 {
    fn code_fingerprint(&mut self,project:&[u8;32],register:bool)->Result<[u8;32],Failure>{
        use sha2::{Digest,Sha256};
        if self.schema_profile!=super::super::NativeSchemaProfile::CodeSource{return Err(Failure::UnsupportedDurability);}
        self.pin_objects_for_grant(register)?;
        let pinned=self.pinned_root.as_ref().ok_or(Failure::CustodyConflict)?;
        if register{
            self.fresh_container.borrow().as_ref().ok_or(Failure::CustodyConflict)?.verify_pinned(pinned)?;
            let c=self.connect()?;
            for table in ["vc04_code_heads","vc04_code_revisions","vc04_code_payloads"]{
                let n:i64=c.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get(0)).map_err(super::super::database_error)?;
                if n!=0{return Err(Failure::CustodyConflict);}
            }
        }
        Ok(root_fingerprint_fields(&current_sid_hash()?,&Sha256::digest(project).into(),&pinned.root_file,&pinned.root_object,&pinned.db_file,&pinned.db_object))
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn register_code_source_grant(&mut self,project:&[u8;32],next:&crate::unit_wire::project_runtime_attachment::code_source::Anchor)->Result<(),Failure>{
        let fingerprint=self.code_fingerprint(project,true)?;let mut nonce=[0;32];getrandom::fill(&mut nonce).map_err(|_|Failure::UnsupportedDurability)?;
        let record=CodeGrantRecord{nonce,fingerprint,project:*project,committed:None,pending:Some(next.clone())};
        WindowsRootGrantStoreV1::write_plaintext(Namespace::CodeSource,&fingerprint,&record.bytes()?,true)?;
        self.fresh_container.borrow_mut().take();Ok(())
    }
    fn read_code_grant(&mut self,project:&[u8;32])->Result<CodeGrantRecord,Failure>{
        let fingerprint=self.code_fingerprint(project,false)?;
        let mut raw=WindowsRootGrantStoreV1::read_plaintext(Namespace::CodeSource,&fingerprint)?;
        let record=CodeGrantRecord::decode(&raw);raw.fill(0);let record=record?;
        if record.fingerprint!=fingerprint||record.project!=*project{return Err(Failure::CustodyConflict);}Ok(record)
    }
    fn write_code_grant(&mut self,before:&CodeGrantRecord,next:&CodeGrantRecord)->Result<(),Failure>{
        let checked=self.read_code_grant(&before.project)?;
        if checked.bytes()?!=before.bytes()?||next.nonce!=before.nonce||next.project!=before.project||next.fingerprint!=before.fingerprint{return Err(Failure::CustodyConflict);}
        WindowsRootGrantStoreV1::write_plaintext(Namespace::CodeSource,&next.fingerprint,&next.bytes()?,false)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn resolve_code_source_grant(&mut self,project:&[u8;32],observed:Option<&crate::unit_wire::project_runtime_attachment::code_source::Anchor>)
        ->Result<Option<crate::unit_wire::project_runtime_attachment::code_source::Anchor>,Failure>{
        let mut record=self.read_code_grant(project)?;
        if let Some(pending)=&record.pending{
            if observed==Some(pending){
                let mut next=CodeGrantRecord::decode(&record.bytes()?)?;next.committed=next.pending.take();self.write_code_grant(&record,&next)?;record=next;
            }else if observed==record.committed.as_ref(){
                if record.committed.is_none(){return Ok(record.pending);}
                let mut next=CodeGrantRecord::decode(&record.bytes()?)?;next.pending=None;self.write_code_grant(&record,&next)?;record=next;
            }else{return Err(Failure::CustodyConflict);}
        }
        if record.committed.as_ref()!=observed{return Err(Failure::CustodyConflict);}Ok(None)
    }
    pub(in crate::snapshot_v2_durable_transaction::native) fn begin_code_source_update(&mut self,project:&[u8;32],expected:Option<&crate::unit_wire::project_runtime_attachment::code_source::Anchor>,next:&crate::unit_wire::project_runtime_attachment::code_source::Anchor)->Result<(),Failure>{
        let record=self.read_code_grant(project)?;
        if record.committed.as_ref()!=expected{return Err(Failure::StaleRevision);}
        if record.pending.as_ref()==Some(next){return Ok(());}
        if record.pending.is_some(){return Err(Failure::CustodyConflict);}
        let mut updated=CodeGrantRecord::decode(&record.bytes()?)?;updated.pending=Some(next.clone());self.write_code_grant(&record,&updated)
    }
}
