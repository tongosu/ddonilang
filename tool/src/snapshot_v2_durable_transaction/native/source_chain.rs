//! Full immutable source-only ancestry; never a first-use permission.
use super::{SqliteAdapterV1,NativeSchemaProfile,Failure,database_error};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
use rusqlite::{params,types::ValueRef};
pub(super) struct SourceChain {pub(super) initial:Source,pub(super) latest:Source}
/// Immutable source ancestry retained behind a world head. This is not the
/// Unused/source-head reader and does not issue a fresh first-use permit.
pub(super) fn read_retained_source_ancestry(db:&rusqlite::Connection,project:&[u8],revision:&[u8])->Result<SourceChain,Failure>{
    let count:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_project_revisions WHERE payload_kind=0 LIMIT 65537)",[],|r|r.get(0)).map_err(database_error)?;
    let payloads:i64=db.query_row("SELECT count(*) FROM (SELECT 1 FROM vc04_prepared_payloads WHERE payload_kind=0 LIMIT 65537)",[],|r|r.get(0)).map_err(database_error)?;
    if !(1..=65536).contains(&count)||count!=payloads{return Err(Failure::CustodyConflict);}
    let latest=read(db,project,revision)?;let mut next=latest.parent().to_vec();let mut seen=1;
    let mut initial=None;
    while !next.is_empty(){
        if seen>=count{return Err(Failure::CustodyConflict);}
        let source=read(db,project,&next)?;seen+=1;
        if source.profile()!=latest.profile(){return Err(Failure::CustodyConflict);}
        next=source.parent().to_vec();if next.is_empty(){initial=Some(source);}
    }
    if seen!=count{return Err(Failure::CustodyConflict);}
    Ok(SourceChain{initial:match initial{Some(s)=>s,None=>read(db,project,revision)?},latest})
}
pub(super) fn read(db:&rusqlite::Connection,project:&[u8],revision:&[u8])->Result<Source,Failure>{
    db.query_row("SELECT r.revision_integrity_sha256,r.payload_kind,r.payload_key,
        p.project_identity,p.expected_parent_history_revision_identity,p.new_history_revision_identity,
        p.execution_profile_identity,p.project_execution_snapshot_identity,p.revision_integrity_sha256,
        p.revision_envelope_bytes,p.project_payload_bytes,p.payload_kind,p.payload_key,
        p.transaction_digest,p.canonical_snapshot_v2_bytes,p.snapshot_v2_byte_length_u64,p.snapshot_v2_sha256
        FROM vc04_project_revisions r JOIN vc04_prepared_payloads p
        ON r.payload_kind=p.payload_kind AND r.payload_key=p.payload_key
        WHERE r.project_identity=?1 AND r.history_revision_identity=?2",
        params![project,revision],|row|{
            let blob=|index,max|->rusqlite::Result<&[u8]>{match row.get_ref(index)?{
                ValueRef::Blob(bytes) if !bytes.is_empty()&&bytes.len()<=max=>Ok(bytes),
                _=>Err(rusqlite::Error::InvalidQuery),
            }};
            let integrity:[u8;32]=blob(0,32)?.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?;
            if blob(2,32)?!=integrity||blob(3,32)?!=project||blob(5,4096)?!=revision
                ||blob(8,32)?!=integrity||blob(12,32)?!=integrity
                ||[1,11].iter().any(|i|row.get::<_,i64>(*i).ok()!=Some(0))
                ||[13,14,15,16].iter().any(|i|!matches!(row.get_ref(*i),Ok(ValueRef::Null))){
                return Err(rusqlite::Error::InvalidQuery);
            }
            let parent=match row.get_ref(4)?{ValueRef::Null=>&[][..],ValueRef::Blob(v) if !v.is_empty()&&v.len()<=4096=>v,
                _=>return Err(rusqlite::Error::InvalidQuery)};
            let pesi:[u8;32]=blob(7,32)?.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?;
            Source::restore(project,parent,revision,&integrity,blob(6,4096)?,&pesi,blob(10,2_000_000)?,blob(9,1_000_000)?)
                .map_err(|_|rusqlite::Error::InvalidQuery)
        }).map_err(database_error)
}
impl SqliteAdapterV1 {
    pub(super) fn read_source_chain(&self,project:&[u8])->Result<SourceChain,Failure>{
        if self.schema_profile!=NativeSchemaProfile::SourceRevision||project.len()!=32{return Err(Failure::UnsupportedDurability);}
        self.confirm_container_durable()?;
        let db=self.connect()?;db.execute_batch("BEGIN").map_err(database_error)?;
        let count=|table:&str|->Result<i64,Failure>{db.query_row(
            &format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 65537)"),[],|r|r.get(0)).map_err(database_error)};
        let revisions=count("vc04_project_revisions")?;
        if !(1..=65536).contains(&revisions)||count("vc04_prepared_payloads")?!=revisions
            ||count("vc04_project_heads")?!=1||count("vc04_recovery_custody_v1")?!=0{return Err(Failure::CustodyConflict);}
        let (head,integrity,key,kind):(Vec<u8>,Vec<u8>,Vec<u8>,i64)=db.query_row(
            "SELECT history_revision_identity,revision_integrity_sha256,payload_key,payload_kind FROM vc04_project_heads WHERE project_identity=?1",
            [project],|r|{
                let blob=|index,max|->rusqlite::Result<Vec<u8>>{match r.get_ref(index)?{
                    ValueRef::Blob(v) if !v.is_empty()&&v.len()<=max=>Ok(v.to_vec()),
                    _=>Err(rusqlite::Error::InvalidQuery)}};
                Ok((blob(0,4096)?,blob(1,32)?,blob(2,32)?,r.get(3)?))
            }).map_err(database_error)?;
        if head.is_empty()||head.len()>4096||integrity.len()!=32||key!=integrity||kind!=0{return Err(Failure::CustodyConflict);}
        let latest=read(&db,project,&head)?;
        if latest.integrity().as_slice()!=integrity{return Err(Failure::CustodyConflict);}
        let mut next=latest.parent().to_vec();let mut seen=1i64;let mut initial=None;
        // With one immutable parent per revision a cycle cannot reach S0.
        // The exact row count bounds traversal without retaining all payloads.
        while !next.is_empty(){
            if seen>=revisions{return Err(Failure::CustodyConflict);}
            let current=read(&db,project,&next)?;
            if current.profile()!=latest.profile(){return Err(Failure::CustodyConflict);}
            seen+=1;next=current.parent().to_vec();
            if next.is_empty(){initial=Some(current);}
        }
        if seen!=revisions{return Err(Failure::CustodyConflict);}
        let initial=match initial{Some(source)=>source,None=>read(&db,project,&head)?};
        db.execute_batch("COMMIT").map_err(database_error)?;
        Ok(SourceChain{initial,latest})
    }
}
