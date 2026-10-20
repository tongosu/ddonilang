//! Exact R1.1 §5.1: bounded mixed revision accounting. This proves SQL/source
//! data against caller-supplied witness bytes, not a root grant or Run permit.
use super::*;
use crate::snapshot_v2_durable_transaction::native::root_grant::source_transition::{Record,Phase};
use crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision as Source;
use std::collections::BTreeSet;
use crate::unit_wire::resumable_complete_state as complete;

struct Revision {revision:Vec<u8>,integrity:[u8;32],key:[u8;32],kind:u8}
pub(in crate::snapshot_v2_durable_transaction::native) struct CheckedMixedRevisionHistory {
    pub initial_source_revision:Vec<u8>,pub initial_source_integrity:[u8;32],
    pub first_runtime_source_revision:Vec<u8>,pub first_runtime_source_integrity:[u8;32],
    pub latest_source_revision:Vec<u8>,pub latest_source_integrity:[u8;32],pub latest_source_pesi:[u8;32],
    pub current_world:Option<StoredPayloadV1>,pub last_world:StoredPayloadV1,
    pub current_segment_digests:Vec<[u8;32]>,pub world_count:u64,pub source_count:u64,
}
fn revision(db:&rusqlite::Connection,project:&[u8],id:&[u8],head:bool)->Result<Revision,Failure>{
    let sql=if head {"SELECT history_revision_identity,revision_integrity_sha256,payload_key,payload_kind FROM vc04_project_heads WHERE project_identity=?1"}
        else {"SELECT history_revision_identity,revision_integrity_sha256,payload_key,payload_kind FROM vc04_project_revisions WHERE project_identity=?1 AND history_revision_identity=?2"};
    let decode=|r:&rusqlite::Row<'_>|->rusqlite::Result<Revision>{
        let id=r.get_ref(0)?.as_blob()?;let integrity=r.get_ref(1)?.as_blob()?;let key=r.get_ref(2)?.as_blob()?;let kind=r.get::<_,i64>(3)?;
        if id.is_empty()||id.len()>4096||integrity.len()!=32||key.len()!=32||!(0..=1).contains(&kind){return Err(rusqlite::Error::InvalidQuery);}
        Ok(Revision{revision:id.to_vec(),integrity:integrity.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?,
            key:key.try_into().map_err(|_|rusqlite::Error::InvalidQuery)?,kind:kind as u8})
    };
    if head {db.query_row(sql,[project],decode)}else{db.query_row(sql,params![project,id],decode)}.map_err(super::super::database_error)
}
fn world(db:&rusqlite::Connection,node:&Revision,project:&[u8])->Result<StoredPayloadV1,Failure>{
    let p=read_world(db,&node.key)?.ok_or(Failure::RecoveryRequired)?;
    if p.binding.project!=project||p.binding.new_revision!=node.revision||p.binding.revision_integrity!=node.integrity {
        return Err(Failure::CustodyConflict);
    }
    Ok(p)
}
fn count(db:&rusqlite::Connection,table:&str,kind:Option<u8>,limit:u64)->Result<u64,Failure>{
    // Only fixed internal names/counters call this helper.
    let sql=format!("SELECT count(*) FROM (SELECT 1 FROM {table} {} LIMIT {})",
        kind.map(|k|format!("WHERE payload_kind={k}")).unwrap_or_default(),limit+1);
    let value:i64=db.query_row(&sql,[],|r|r.get(0)).map_err(super::super::database_error)?;
    let value=u64::try_from(value).map_err(|_|Failure::LimitExceeded)?;
    if value>limit{return Err(Failure::LimitExceeded);}Ok(value)
}
pub(in crate::snapshot_v2_durable_transaction::native) fn check_mixed_source_world_history(db:&rusqlite::Connection,project:&[u8],epoch:u64,
    current_digest:&[u8;32],request_hash:&[u8;32],token_hash:&[u8;32],witness_hash:&[u8;32],transition:Option<&Record>)
    ->Result<CheckedMixedRevisionHistory,Failure>{
    if project.len()!=32||epoch==0||epoch>65536{return Err(Failure::CustodyConflict);}
    let head=revision(db,project,&[],true)?;
    let head_kind=head.kind;let head_revision=head.revision.clone();
    let mut nodes=Vec::new();let mut next=head.revision.clone();let mut seen=BTreeSet::new();
    while !next.is_empty(){
        if nodes.len()>=65536||!seen.insert(next.clone()){return Err(Failure::LimitExceeded);}
        let node=revision(db,project,&next,false)?;
        next=if node.kind==0 {
            let s=super::super::source_chain::read(db,project,&node.revision)?;
            if node.key!=*s.integrity()||node.integrity!=*s.integrity(){return Err(Failure::CustodyConflict);}s.parent().to_vec()
        }else{world(db,&node,project)?.binding.expected_parent};
        nodes.push(node);
    }
    nodes.reverse();let first=nodes.first().ok_or(Failure::CustodyConflict)?;
    if first.kind!=0||nodes.last().is_none_or(|n|n.revision!=head_revision||n.kind!=head_kind||n.key!=head.key||n.integrity!=head.integrity){return Err(Failure::CustodyConflict);}
    let initial=super::super::source_chain::read(db,project,&first.revision)?;
    let initial_revision=initial.revision().to_vec();let initial_integrity=*initial.integrity();
    let profile=initial.profile().to_vec();let mut latest_source=initial;
    let mut first_runtime=None;let mut segment_digests=Vec::new();let mut segment_worlds=0u64;
    let mut source_count=0u64;let mut world_count=0u64;let mut segments=0u64;
    let mut last_world:Option<StoredPayloadV1>=None;let mut authoring_anchor:Option<(StoredPayloadV1,u64)>=None;
    let mut predecessor_node:Option<&Revision>=None;
    for node in &nodes {
        if node.kind==0 {
            let source=super::super::source_chain::read(db,project,&node.revision)?;
            if source.profile()!=profile{return Err(Failure::CustodyConflict);}
            if let Some(old)=predecessor_node {
                if source.parent()!=old.revision{return Err(Failure::CustodyConflict);}
                if old.kind==1 {
                    authoring_anchor=Some((last_world.as_ref().ok_or(Failure::CustodyConflict)?.clone(),world_count));
                    segment_digests.clear();segment_worlds=0;
                }
                match (latest_source.owned().registration(),source.owned().registration()){
                    (Some(old),Some(new))=>new.verify_retained_from(old).map_err(|_|Failure::CustodyConflict)?,
                    (Some(_),None)=>return Err(Failure::CustodyConflict),_=>{},
                }
            }else if !source.parent().is_empty(){return Err(Failure::CustodyConflict);}
            source_count=source_count.checked_add(1).ok_or(Failure::LimitExceeded)?;latest_source=source;
        }else {
            let p=world(db,node,project)?;
            let old=predecessor_node.ok_or(Failure::CustodyConflict)?;
            if p.binding.expected_parent!=old.revision||p.binding.pesi!=*latest_source.pesi()||p.binding.execution_profile!=profile{return Err(Failure::CustodyConflict);}
            let envelope=Source::prepare(latest_source.owned(),&p.binding.expected_parent,&p.binding.new_revision).map_err(|_|Failure::CustodyConflict)?;
            if envelope.envelope()!=p.envelope||envelope.integrity()!=&node.integrity{return Err(Failure::CustodyConflict);}
            let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&p.snapshot,latest_source.pesi())?;
            let descriptor=complete::rebuild_source_descriptor_candidate(layout.supplies[9],latest_source.owned())?;
            if u64::try_from(descriptor.prior_durable_receipts.len()).ok()!=Some(segment_worlds){return Err(Failure::CustodyConflict);}
            for (index,(receipt,digest)) in descriptor.prior_durable_receipts.iter().zip(&segment_digests).enumerate(){
                let receipt=crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(receipt)?;
                let retained=receipt.native_head()?;let old=read_world(db,digest)?.ok_or(Failure::RecoveryRequired)?;
                receipt.check_binding(&old.binding,b"vc04.sqlite-durability.r1")?;
                if retained.project!=project||retained.digest!=*digest||retained.revision!=old.binding.new_revision||retained.integrity!=old.binding.revision_integrity{return Err(Failure::CustodyConflict);}
                let previous_layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&old.snapshot,latest_source.pesi())?;
                let previous=complete::rebuild_source_descriptor_candidate(previous_layout.supplies[9],latest_source.owned())?;
                if previous.execution_instance!=descriptor.execution_instance||previous.source_descriptor_prefix!=descriptor.source_descriptor_prefix
                    ||previous.prior_durable_receipts.len()!=index||previous.prior_durable_receipts!=descriptor.prior_durable_receipts[..index]{return Err(Failure::CustodyConflict);}
            }
            let current=&p.digest==current_digest;
            let anchor=transition.filter(|t|t.anchor_digest==p.digest);
            if current&&anchor.is_some_and(|t|t.anchor_record!=*witness_hash){return Err(Failure::CustodyConflict);}
            check_world_custody_for_recorded_source(db,&p,&old.key,latest_source.owned(),latest_source.pesi(),
                current.then_some(request_hash),current.then_some(token_hash),if current{Some(witness_hash)}else{anchor.map(|t|&t.anchor_record)})?;
            if segment_worlds==0 {segments=segments.checked_add(1).ok_or(Failure::LimitExceeded)?;
                if first_runtime.is_none(){first_runtime=Some((latest_source.revision().to_vec(),*latest_source.integrity()));}}
            segment_worlds=segment_worlds.checked_add(1).ok_or(Failure::LimitExceeded)?;
            world_count=world_count.checked_add(1).ok_or(Failure::LimitExceeded)?;segment_digests.push(p.digest);last_world=Some(p);
        }
        predecessor_node=Some(node);
    }
    let last=last_world.ok_or(Failure::RecoveryRequired)?;
    if last.digest!=*current_digest||world_count!=epoch||count(db,"vc04_project_heads",None,1)?!=1
        ||count(db,"vc04_project_revisions",Some(0),65536)?!=source_count||count(db,"vc04_prepared_payloads",Some(0),65536)?!=source_count
        ||count(db,"vc04_project_revisions",Some(1),65536)?!=world_count||count(db,"vc04_prepared_payloads",Some(1),65536)?!=world_count
        ||count(db,"vc04_recovery_custody_v1",None,196608)?!=world_count.checked_mul(3).ok_or(Failure::LimitExceeded)? {return Err(Failure::CustodyConflict);}
    match transition {
        None=>if segments!=1||head_kind!=1||authoring_anchor.is_some(){return Err(Failure::RecoveryRequired);},
        Some(t)=>{
            let (anchor,a)=authoring_anchor.as_ref().ok_or(Failure::CustodyConflict)?;
            if t.anchor_digest!=anchor.digest||t.anchor_revision!=anchor.binding.new_revision||t.anchor_integrity!=anchor.binding.revision_integrity||t.anchor_epoch!=*a
                ||t.source_revision!=latest_source.revision()||t.source_integrity!=*latest_source.integrity()||t.source_pesi!=*latest_source.pesi(){return Err(Failure::CustodyConflict);}
            let s_index=nodes.iter().position(|n|n.revision==t.source_revision).ok_or(Failure::CustodyConflict)?;
            let parent=s_index.checked_sub(1).and_then(|i|nodes.get(i)).ok_or(Failure::CustodyConflict)?;
            if t.parent_revision!=parent.revision||t.parent_integrity!=parent.integrity||t.parent_kind!=parent.kind{return Err(Failure::CustodyConflict);}
            match t.phase {
                Phase::Ready=>if head_kind!=0||segment_worlds!=0||epoch!=*a||t.bound_digest!=[0;32]||t.first_bound_epoch!=0{return Err(Failure::RecoveryRequired);},
                Phase::Bound=>if head_kind!=1||segment_worlds==0||t.first_bound_epoch!=a.checked_add(1).ok_or(Failure::LimitExceeded)?
                    ||segment_digests.first()!=Some(&t.bound_digest)||a.checked_add(segment_worlds)!=Some(epoch){return Err(Failure::CustodyConflict);},
                _=>return Err(Failure::RecoveryRequired),
            }
        }
    }
    let (first_runtime_source_revision,first_runtime_source_integrity)=first_runtime.ok_or(Failure::CustodyConflict)?;
    Ok(CheckedMixedRevisionHistory{initial_source_revision:initial_revision,initial_source_integrity:initial_integrity,
        first_runtime_source_revision,first_runtime_source_integrity,latest_source_revision:latest_source.revision().to_vec(),latest_source_integrity:*latest_source.integrity(),
        latest_source_pesi:*latest_source.pesi(),current_world:(head_kind==1).then(||last.clone()),last_world:last,current_segment_digests:segment_digests,world_count,source_count})
}

#[cfg(all(test,windows))]
mod tests {
    use super::*;
    use sha2::{Digest,Sha256};
    #[test]
    fn mixed_reader_preserves_actual_native_first_segment_and_rejects_wrong_witness(){
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let mut nonce=[0;32];getrandom::fill(&mut nonce).unwrap();
        let root=std::env::temp_dir().join(format!("ddn-source-restart-history-{}",hex::encode(nonce)));std::fs::create_dir(&root).unwrap();
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
            SourceFile{path:"root.ddn".into(),text:"(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.".into()},
            SourceFile{path:"main.ddn".into(),text:"채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. }. }.".into()}]},String::new()).unwrap();
        let saved=crate::runtime_surface::save_initial_authored_source_native(&project.save().unwrap(),&[1],&root).unwrap();
        let retained=crate::runtime_surface::begin_first_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap(),&saved.revision_identity,&saved.revision_integrity).unwrap()
            .execute(1,0,None,None).unwrap().prepare(&[2]).unwrap().write_prepared().unwrap().commit().unwrap();drop(retained);
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let digest:Vec<u8>=db.query_row("SELECT payload_key FROM vc04_project_heads",[],|r|r.get(0)).unwrap();let digest:[u8;32]=digest.try_into().unwrap();
        let request:Vec<u8>=db.query_row("SELECT admitted_request_bytes FROM vc04_recovery_custody_v1 WHERE journal_ordinal=0",[],|r|r.get(0)).unwrap();
        let record:Vec<u8>=db.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE journal_ordinal=2",[],|r|r.get(0)).unwrap();
        let request_hash=Sha256::digest(request).into();let mut token=vec![1];token.extend(digest);let token_hash=Sha256::digest(token).into();let record_hash=Sha256::digest(record).into();
        let checked=check_mixed_source_world_history(&db,&saved.project_identity,1,&digest,&request_hash,&token_hash,&record_hash,None).unwrap();
        assert_eq!(checked.world_count,1);assert_eq!(checked.source_count,1);assert_eq!(checked.current_segment_digests,vec![digest]);assert_eq!(checked.first_runtime_source_revision,vec![1]);
        assert!(check_mixed_source_world_history(&db,&saved.project_identity,2,&digest,&request_hash,&token_hash,&record_hash,None).is_err());
        assert!(check_mixed_source_world_history(&db,&saved.project_identity,1,&digest,&[0;32],&token_hash,&record_hash,None).is_err());
        let old_world=read_world(&db,&digest).unwrap().unwrap();drop(db);
        let original=project.save().unwrap();let locator=saved.grant_locator.as_deref().unwrap();
        let selected=crate::runtime_surface::native_authoring_selection(&root,&saved.project_identity,locator,&original).unwrap();
        let mut snapshot=project.source_snapshot.clone();snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap().text.replace("횟수 + 1","횟수 + 2");
        let changed=crate::authored_project::operate("update",&serde_json::json!({"original_json":original,"source_snapshot":snapshot}).to_string()).unwrap();
        let source=crate::runtime_surface::save_changed_authored_source_native(&changed,&original,&[3],&root,locator,&selected).unwrap();
        assert_eq!(source.revision_identity,vec![3]);
        let cold=crate::runtime_surface::read_changed_authored_source_native(&root,&saved.project_identity,locator,&changed).unwrap();
        assert_eq!(cold.revision_identity,vec![3]);assert!(cold.project_payload_utf8.contains("횟수 + 2"));
        // A stale pre-edit selection cannot overwrite the new source head.
        assert!(crate::runtime_surface::save_changed_authored_source_native(&original,&changed,&[4],&root,locator,&selected).is_err());
        let second=crate::runtime_surface::native_authoring_selection(&root,&saved.project_identity,locator,&changed).unwrap();
        let mut snapshot=project.source_snapshot.clone();snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=snapshot.files.iter().find(|f|f.path=="main.ddn").unwrap().text.replace("횟수 + 1","횟수 + 3");
        let changed_again=crate::authored_project::operate("update",&serde_json::json!({"original_json":changed,"source_snapshot":snapshot}).to_string()).unwrap();
        crate::runtime_surface::save_changed_authored_source_native(&changed_again,&changed,&[4],&root,locator,&second).unwrap();
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        assert_eq!(read_world(&db,&digest).unwrap().unwrap(),old_world);
        let after_record:Vec<u8>=db.query_row("SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 WHERE journal_ordinal=2",[],|r|r.get(0)).unwrap();
        assert_eq!(<[u8;32]>::from(Sha256::digest(after_record)),record_hash);
        assert_eq!(count(&db,"vc04_prepared_payloads",Some(0),65536).unwrap(),3);
        assert_eq!(count(&db,"vc04_prepared_payloads",Some(1),65536).unwrap(),1);
        drop(db);
        let ready=crate::runtime_surface::native_authoring_selection(&root,&saved.project_identity,locator,&changed_again).unwrap();
        let session=crate::runtime_surface::begin_new_authored_source_native(&root,&saved.project_identity,locator,&changed_again,&ready).unwrap();
        // The one-shot protected consumption precedes creation of the new VM.
        assert!(crate::runtime_surface::begin_new_authored_source_native(&root,&saved.project_identity,locator,&changed_again,&ready).is_err());
        let fresh=session.execute(1,0,None,None).unwrap().prepare(&[5]).unwrap().write_prepared().unwrap().commit().unwrap();
        assert_eq!(fresh.summary().unwrap()["output_display"]["texts"],serde_json::json!(["3"]));
        let fresh_artifact=fresh.saved_runtime_artifact().unwrap().to_owned();
        drop(fresh);
        assert!(crate::runtime_surface::begin_new_authored_source_native(&root,&saved.project_identity,locator,&changed_again,&ready).is_err());
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        assert_eq!(read_world(&db,&digest).unwrap().unwrap(),old_world);
        assert_eq!(count(&db,"vc04_prepared_payloads",Some(1),65536).unwrap(),2);
        drop(db);
        let cold=crate::runtime_surface::resume_first_saved_source_world_native(&fresh_artifact,&[4],&[5],&root,locator,1).unwrap();
        assert_eq!(cold["output_display"]["texts"],serde_json::json!(["3","6"]));
        let mut cache=None;
        let continued=crate::runtime_surface::save_continued_source_world_native(&fresh_artifact,&[4],&[5],&[6],&root,locator,1,|a,_|{cache=Some(a.to_owned());Ok(())}).unwrap();
        assert_eq!(continued["output_display"]["texts"],serde_json::json!(["3","6"]));
        let continued_artifact=cache.unwrap();
        let cold=crate::runtime_surface::resume_first_saved_source_world_native(&continued_artifact,&[5],&[6],&root,locator,0).unwrap();
        assert_eq!(cold["state_hash"],continued["state_hash"]);
        // Second real W -> S -> fresh W cycle: global epoch is not local q+1.
        let selected=crate::runtime_surface::native_authoring_selection(&root,&saved.project_identity,locator,&changed_again).unwrap();
        let mut snapshot=project.source_snapshot.clone();let main=snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap();main.text=main.text.replace("횟수 + 1","횟수 + 4");
        let changed_third=crate::authored_project::operate("update",&serde_json::json!({"original_json":changed_again,"source_snapshot":snapshot}).to_string()).unwrap();
        crate::runtime_surface::save_changed_authored_source_native(&changed_third,&changed_again,&[7],&root,locator,&selected).unwrap();
        let selected=crate::runtime_surface::native_authoring_selection(&root,&saved.project_identity,locator,&changed_third).unwrap();
        let second=crate::runtime_surface::begin_new_authored_source_native(&root,&saved.project_identity,locator,&changed_third,&selected).unwrap()
            .execute(1,0,None,None).unwrap().prepare(&[8]).unwrap().write_prepared().unwrap().commit().unwrap();
        assert_eq!(second.summary().unwrap()["output_display"]["texts"],serde_json::json!(["4"]));let second_artifact=second.saved_runtime_artifact().unwrap().to_owned();drop(second);
        let cold=crate::runtime_surface::resume_first_saved_source_world_native(&second_artifact,&[7],&[8],&root,locator,1).unwrap();
        assert_eq!(cold["output_display"]["texts"],serde_json::json!(["4","8"]));
        let db=rusqlite::Connection::open_with_flags(root.join("vc04_snapshot_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();assert_eq!(read_world(&db,&digest).unwrap().unwrap(),old_world);
        assert_eq!(count(&db,"vc04_prepared_payloads",Some(0),65536).unwrap(),4);assert_eq!(count(&db,"vc04_prepared_payloads",Some(1),65536).unwrap(),4);
    }
}
