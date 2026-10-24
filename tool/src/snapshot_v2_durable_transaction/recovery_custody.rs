//! Native R2/R3 child codec under the common Rust transaction owner.
//! These are byte candidates, never source/root/start/Witness/receipt rights.
use super::{SnapshotV2DurabilityFailureV1 as Failure,SnapshotV2TransactionStateV1 as State,TransactionBindingV1};

pub(super) const RECORD_MAX:usize=12_589;
pub(super) const REQUEST_MAX:usize=3_016_737;
const PROFILE:&[u8]=b"vc04.sqlite-durability.r1";
const MAX_FIELDS:[usize;14]=[32,1,4096,4096,4096,32,8,32,32,32,25,33,2,1];

/// Initial row bytes owned by the admitted execution request. This is not a
/// transaction state, start grant, Witness or receipt.
#[cfg(not(target_arch="wasm32"))]
pub(super) struct InitialCustodyRowV1 {
    digest:[u8;32],
    request:std::sync::Arc<[u8]>,
    record:Vec<u8>,
    hash:[u8;32],
}

#[cfg(not(target_arch="wasm32"))]
impl InitialCustodyRowV1 {
    pub(super) fn from_actual_request(request:&super::ActualPreparedRequestV1)->Result<Self,Failure> {
        if !request.binding().expected_parent.is_empty() {return Err(Failure::UnsupportedDurability);}
        if request.raw().len()>REQUEST_MAX {return Err(Failure::LimitExceeded);}
        let digest=request.binding().transaction_digest()?;
        let record=encode_prepared_candidate(request.binding(),None)?;
        let raw=request.shared_raw();
        let hash=Self::row_hash(&digest,0,&[0;32],&raw,&record)?;
        Ok(Self{digest,request:raw,record,hash})
    }
    pub(super) fn row_hash(digest:&[u8;32],ordinal:u8,previous:&[u8;32],request:&[u8],record:&[u8])->Result<[u8;32],Failure> {
        use sha2::{Digest,Sha256};
        if request.is_empty() || request.len()>REQUEST_MAX || record.is_empty() || record.len()>RECORD_MAX {return Err(Failure::LimitExceeded);}
        let mut hash=Sha256::new();
        hash.update(b"DDN-NATIVE-RECOVERY-CUSTODY-ROW-V1\0");hash.update(1u16.to_be_bytes());
        hash.update(digest);hash.update([ordinal]);hash.update(previous);
        hash.update((request.len() as u32).to_be_bytes());hash.update(request);
        hash.update((record.len() as u32).to_be_bytes());hash.update(record);
        Ok(hash.finalize().into())
    }
    pub(super) fn check_request(&self,request:&super::ActualPreparedRequestV1)->Result<(),Failure> {
        if self.digest!=request.binding().transaction_digest()? || self.request.as_ref()!=request.raw()
            || !request.binding().expected_parent.is_empty() {return Err(Failure::CustodyConflict);}
        if self.record!=encode_prepared_candidate(request.binding(),None)?
            || self.hash!=Self::row_hash(&self.digest,0,&[0;32],&self.request,&self.record)? {return Err(Failure::CustodyConflict);}
        Ok(())
    }
    pub(super) fn digest(&self)->&[u8;32] {&self.digest}
    pub(super) fn request(&self)->&[u8] {&self.request}
    pub(super) fn record(&self)->&[u8] {&self.record}
    pub(super) fn hash(&self)->&[u8;32] {&self.hash}
}

/// Queued only from the common loop's actual Verified evidence. No public
/// constructor, state BOOL, SQL return or prospective terminal state is used.
#[cfg(not(target_arch="wasm32"))]
pub(super) struct VerifiedCustodyRowV1 {record:Vec<u8>,hash:[u8;32]}

#[cfg(not(target_arch="wasm32"))]
impl VerifiedCustodyRowV1 {
    pub(super) fn from_verified(evidence:&super::TransactionEvidenceV1,initial:&InitialCustodyRowV1,
        request:&super::ActualPreparedRequestV1)->Result<Self,Failure> {
        initial.check_request(request)?;
        if evidence.state()!=State::Verified || evidence.binding!=*request.binding()
            || evidence.head_changed || evidence.container_confirmed {return Err(Failure::CustodyConflict);}
        let record=Self::verified_record(initial)?;
        let hash=InitialCustodyRowV1::row_hash(initial.digest(),1,initial.hash(),initial.request(),&record)?;
        Ok(Self{record,hash})
    }
    fn verified_record(initial:&InitialCustodyRowV1)->Result<Vec<u8>,Failure> {
        encode_verified_candidate(initial.record())
    }
    pub(super) fn check_initial(&self,initial:&InitialCustodyRowV1,request:&super::ActualPreparedRequestV1)->Result<(),Failure> {
        initial.check_request(request)?;
        if self.record!=Self::verified_record(initial)?
            || self.hash!=InitialCustodyRowV1::row_hash(initial.digest(),1,initial.hash(),initial.request(),&self.record)? {
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
    pub(super) fn record(&self)->&[u8] {&self.record}
    pub(super) fn hash(&self)->&[u8;32] {&self.hash}
}

/// Same fourteen-field codec for either typed parent producer. This only
/// encodes data; actual Verified evidence remains required by the caller.
pub(super) fn encode_verified_candidate(prepared:&[u8])->Result<Vec<u8>,Failure>{
    let candidate=RecoveryRecordCandidateV1::decode(prepared)?;
    if candidate.state!=State::Prepared||candidate.cause!=Failure::Interrupted||candidate.confirmed{return Err(Failure::CustodyConflict);}
    let mut fields=candidate.fields;let phase=[State::Verified as u8];fields[1]=&phase;
    RecoveryRecordCandidateV1::from_fields(fields)?.encode_canonical()
}

/// Encoded only from this execution's sealed outcome, never from an adapter
/// status or a prospective terminal phase. Persistence is a separate gate.
#[cfg(windows)]
pub(super) struct ActualOutcomeCustodyRowV1 {
    ordinal:u8,
    previous:[u8;32],
    record:Vec<u8>,
    hash:[u8;32],
}

#[cfg(windows)]
impl ActualOutcomeCustodyRowV1 {
    pub(super) fn from_outcome(actual:&super::retained_attempt::UnretainedCustodyOutcomeV1)->Result<Self,Failure> {
        actual.initial.check_request(actual.start.request())?;
        let (ordinal,previous)=if let Some(verified)=&actual.issued_verified {
            verified.check_initial(&actual.initial,actual.start.request())?;
            (2,*verified.hash())
        }else{
            if actual.append_completed.is_some() {return Err(Failure::CustodyConflict);}
            (1,*actual.initial.hash())
        };
        let record=encode_actual_outcome_candidate(&actual.owner,actual.initial.record())?;
        let hash=InitialCustodyRowV1::row_hash(actual.initial.digest(),ordinal,&previous,actual.initial.request(),&record)?;
        Ok(Self{ordinal,previous,record,hash})
    }
    pub(super) fn check_outcome(&self,actual:&super::retained_attempt::UnretainedCustodyOutcomeV1)->Result<(),Failure> {
        let expected=Self::from_outcome(actual)?;
        if self.ordinal!=expected.ordinal || self.previous!=expected.previous
            || self.record!=expected.record || self.hash!=expected.hash {return Err(Failure::CustodyConflict);}
        Ok(())
    }
    pub(super) fn ordinal(&self)->u8 {self.ordinal}
    pub(super) fn previous(&self)->&[u8;32] {&self.previous}
    pub(super) fn record(&self)->&[u8] {&self.record}
    pub(super) fn hash(&self)->&[u8;32] {&self.hash}
}

/// Borrowed and bounded only. Even a valid Committed tag is not evidence.
#[derive(Debug)]
pub(super) struct RecoveryRecordCandidateV1<'a> {
    fields:[&'a [u8];14],
    state:State,
    cause:Failure,
    confirmed:bool,
}

fn state(byte:u8)->Result<State,Failure> {
    Ok(match byte {1=>State::Prepared,2=>State::DurablyWritten,3=>State::Verified,
        4=>State::Committed,5=>State::Aborted,6=>State::RecoveryRequired,_=>return Err(Failure::CorruptRecoveryRecord)})
}
fn cause(code:u16)->Result<Failure,Failure> {
    use Failure::*;
    Ok(match code {1=>InvalidInput,2=>LimitExceeded,3=>SnapshotHashMismatch,4=>AtomEvidenceInvalid,
        5=>SubjectCustodyInvalid,6=>PesiMismatch,7=>ForeignPesi,8=>StaleRevision,9=>CustodyConflict,
        10=>ConflictingRetry,11=>UnsupportedDurability,12=>WriteFailed,13=>ShortWrite,14=>SyncFailed,
        15=>RereadFailed,16=>RereadMismatch,17=>AtomicCommitFailed,18=>ContainerDurabilityFailed,
        19=>RevisionCommitFailed,20=>AdapterAborted,21=>Interrupted,22=>CorruptRecoveryRecord,
        23=>RecoveryRequired,24=>ArtifactMismatch,_=>return Err(CorruptRecoveryRecord)})
}

impl<'a> RecoveryRecordCandidateV1<'a> {
    pub(super) fn decode(raw:&'a [u8])->Result<Self,Failure> {
        if raw.len()>RECORD_MAX {return Err(Failure::LimitExceeded);}
        if raw.first()!=Some(&14) {return Err(Failure::CorruptRecoveryRecord);}
        let mut cursor=1usize;let mut fields:[&[u8];14]=[&[];14];
        for (i,max) in MAX_FIELDS.into_iter().enumerate() {
            if raw.get(cursor)!=Some(&(i as u8+1)) {return Err(Failure::CorruptRecoveryRecord);}
            let length=u32::from_be_bytes(raw.get(cursor+1..cursor+5).ok_or(Failure::CorruptRecoveryRecord)?
                .try_into().map_err(|_|Failure::CorruptRecoveryRecord)?) as usize;
            if length>max {return Err(Failure::LimitExceeded);}
            cursor=cursor.checked_add(5).ok_or(Failure::LimitExceeded)?;
            let end=cursor.checked_add(length).ok_or(Failure::LimitExceeded)?;
            fields[i]=raw.get(cursor..end).ok_or(Failure::CorruptRecoveryRecord)?;cursor=end;
        }
        if cursor!=raw.len() {return Err(Failure::CorruptRecoveryRecord);}
        Self::from_fields(fields)
    }

    fn from_fields(fields:[&'a [u8];14])->Result<Self,Failure> {
        for (i,field) in fields.iter().enumerate() {if field.len()>MAX_FIELDS[i] {return Err(Failure::LimitExceeded);}}
        for (i,length) in [(0,32),(1,1),(5,32),(6,8),(7,32),(8,32),(11,33),(12,2),(13,1)] {
            if fields[i].len()!=length {return Err(Failure::CorruptRecoveryRecord);}
        }
        if fields[2].is_empty() || fields[4].is_empty() || ![0,32].contains(&fields[9].len())
            || fields[11][0]!=1 || fields[11][1..]!=*fields[0] {
            return Err(Failure::CorruptRecoveryRecord);
        }
        if fields[10]!=PROFILE {return Err(Failure::UnsupportedDurability);}
        let snapshot_length=u64::from_be_bytes(fields[6].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?);
        if snapshot_length==0 || snapshot_length>2_000_000 {return Err(Failure::LimitExceeded);}
        let confirmed=match fields[13] {[0]=>false,[1]=>true,_=>return Err(Failure::CorruptRecoveryRecord)};
        Ok(Self {state:state(fields[1][0])?,cause:cause(u16::from_be_bytes(fields[12].try_into().map_err(|_|Failure::CorruptRecoveryRecord)?))?,confirmed,fields})
    }

    pub(super) fn encode_canonical(&self)->Result<Vec<u8>,Failure> {
        let length=self.fields.iter().try_fold(1usize,|n,f|n.checked_add(5)?.checked_add(f.len())).ok_or(Failure::LimitExceeded)?;
        if length>RECORD_MAX {return Err(Failure::LimitExceeded);}
        let mut raw=Vec::new();raw.try_reserve_exact(length).map_err(|_|Failure::LimitExceeded)?;raw.push(14);
        for (i,field) in self.fields.iter().enumerate(){raw.push(i as u8+1);raw.extend((field.len() as u32).to_be_bytes());raw.extend(*field);}
        Ok(raw)
    }

    pub(super) fn state(&self)->State {self.state}
    pub(super) fn cause(&self)->Failure {self.cause}
    pub(super) fn confirmed(&self)->bool {self.confirmed}

    /// Mechanical equality only; the input binding/head still needs its source
    /// and actual common-owner provenance before custody can be admitted.
    pub(super) fn check_binding(&self,binding:&TransactionBindingV1,predecessor:Option<&[u8;32]>)->Result<(),Failure> {
        let digest=binding.transaction_digest()?;let length=binding.snapshot_length.to_be_bytes();
        if binding.expected_parent.is_empty()!=predecessor.is_none()
            || self.fields[0]!=digest || self.fields[2]!=binding.project || self.fields[3]!=binding.expected_parent
            || self.fields[4]!=binding.new_revision || self.fields[5]!=binding.pesi || self.fields[6]!=length
            || self.fields[7]!=binding.snapshot_sha256 || self.fields[8]!=binding.revision_integrity
            || self.fields[9]!=predecessor.map(|p|p.as_slice()).unwrap_or(&[]) {
            return Err(Failure::CustodyConflict);
        }
        Ok(())
    }
}

/// Data encoder only. The R3.1 admitted issuer must still supply real source,
/// parent evidence, same-root grant and exact request before SQL publication.
pub(super) fn encode_prepared_candidate(binding:&TransactionBindingV1,predecessor:Option<&[u8;32]>)->Result<Vec<u8>,Failure> {
    let digest=binding.transaction_digest()?;let length=binding.snapshot_length.to_be_bytes();
    let mut token=[0u8;33];token[0]=1;token[1..].copy_from_slice(&digest);
    let initial_cause=(Failure::Interrupted as u16).to_be_bytes();
    let fields:[&[u8];14]=[&digest,&[State::Prepared as u8],&binding.project,&binding.expected_parent,&binding.new_revision,
        &binding.pesi,&length,&binding.snapshot_sha256,&binding.revision_integrity,predecessor.map(|p|p.as_slice()).unwrap_or(&[]),
        PROFILE,&token,&initial_cause,&[0]];
    let candidate=RecoveryRecordCandidateV1::from_fields(fields)?;candidate.check_binding(binding,predecessor)?;candidate.encode_canonical()
}

/// Preserve the actual loop's phase and original cause. This returns bytes,
/// NOT a witness: SQL retention/root/request/epoch admission remains mandatory.
#[cfg(not(target_arch="wasm32"))]
pub(super) fn encode_actual_outcome_candidate(outcome:&super::NativeOwnerOutcomeV1,initial:&[u8])->Result<Vec<u8>,Failure> {
    let original=RecoveryRecordCandidateV1::decode(initial)?;
    if original.state!=State::Prepared || original.cause!=Failure::Interrupted || original.confirmed {
        return Err(Failure::CustodyConflict);
    }
    let predecessor=if original.fields[9].is_empty(){None}else{Some(<&[u8;32]>::try_from(original.fields[9]).map_err(|_|Failure::CorruptRecoveryRecord)?)};
    let evidence=outcome.evidence();original.check_binding(&evidence.binding,predecessor)?;
    let cause=if let Some(failure)=outcome.failure() {
        if failure.state!=Some(evidence.state()) || !matches!(evidence.state(),State::Aborted|State::RecoveryRequired) {
            return Err(Failure::CustodyConflict);
        }
        // Runtime-only successor errors have no assigned R1 recovery tag.
        if failure.cause as u16>24 {return Err(Failure::UnsupportedDurability);}
        failure.cause
    } else {
        if evidence.state()!=State::Committed {return Err(Failure::CustodyConflict);}
        // Retain the original record's placeholder; do not invent a historical
        // failure for a transaction whose actual outcome is Committed.
        original.cause
    };
    let phase=[evidence.state() as u8];let cause=(cause as u16).to_be_bytes();let confirmation=[u8::from(evidence.container_confirmed)];
    let mut fields=original.fields;fields[1]=&phase;fields[12]=&cause;fields[13]=&confirmation;
    RecoveryRecordCandidateV1::from_fields(fields)?.encode_canonical()
}

#[cfg(not(target_arch="wasm32"))]
pub(super) fn check_actual_outcome_candidate(raw:&[u8],outcome:&super::NativeOwnerOutcomeV1,initial:&[u8])->Result<(),Failure> {
    let expected=encode_actual_outcome_candidate(outcome,initial)?;
    if raw!=expected {return Err(Failure::CustodyConflict);}Ok(())
}

/// Streaming verifier for the currently implemented INITIAL producer only.
/// The SQL visitor borrows one row at a time; no prefix Vec or last-wins rule.
/// A successful result remains historical data, not new transaction evidence.
#[cfg(windows)]
pub(super) struct InitialPrefixVerifierV1<'a> {
    expected:&'a super::retained_attempt::ExplicitRecoveryRequestV1,
    witness_hash:[u8;32],
    expected_epoch:u64,
    outcomes:u64,
    next:u8,
    previous:[u8;32],
    terminal:Option<LoadedInitialCustodyV1>,
}

#[cfg(windows)]
pub(super) struct LoadedInitialCustodyV1 {
    record:Vec<u8>,
    state:State,
    cause:Failure,
    ordinal:u8,
    row_hash:[u8;32],
}

#[cfg(windows)]
impl LoadedInitialCustodyV1 {
    pub(super) fn record(&self)->&[u8] {&self.record}
    pub(super) fn state(&self)->State {self.state}
    pub(super) fn cause(&self)->Failure {self.cause}
    pub(super) fn ordinal(&self)->u8 {self.ordinal}
    pub(super) fn row_hash(&self)->&[u8;32] {&self.row_hash}
}

#[cfg(windows)]
impl<'a> InitialPrefixVerifierV1<'a> {
    pub(super) fn new(expected:&'a super::retained_attempt::ExplicitRecoveryRequestV1,witness_hash:[u8;32],expected_epoch:u64)->Self {
        Self{expected,witness_hash,expected_epoch,outcomes:0,next:0,previous:[0;32],terminal:None}
    }
    pub(super) fn accept(&mut self,ordinal:u8,previous:&[u8],request:&[u8],record:&[u8],hash:&[u8])->Result<(),Failure> {
        if ordinal!=self.next || previous!=self.previous || request!=self.expected.raw() {
            return Err(Failure::CustodyConflict);
        }
        let payload=self.expected.payload();
        let candidate=RecoveryRecordCandidateV1::decode(record)?;
        candidate.check_binding(&payload.binding,None)?;
        if candidate.encode_canonical()?!=record {return Err(Failure::CorruptRecoveryRecord);}
        let computed=InitialCustodyRowV1::row_hash(&payload.digest,ordinal,&self.previous,request,record)?;
        if hash!=computed {return Err(Failure::CustodyConflict);}
        if ordinal==0 {
            if record!=encode_prepared_candidate(&payload.binding,None)? {return Err(Failure::CustodyConflict);}
        }else if ordinal==1 && self.terminal.is_none() && candidate.state()==State::Verified {
            // The live producer retains Prepared and then Verified after the
            // actual DurablyWritten step. Do not invent a new state edge.
            if candidate.cause()!=Failure::Interrupted || candidate.confirmed() {return Err(Failure::CustodyConflict);}
        }else{
            if let Some(prior)=&self.terminal {
                // Each retained stateful recovery has one outcome row. A
                // terminal C/A cannot be silently reopened by a later row.
                if prior.state!=State::RecoveryRequired || !prior.state.permits(candidate.state())
                    || candidate.cause()!=prior.cause {return Err(Failure::CustodyConflict);}
            }else{
                if !matches!((ordinal,candidate.state()),(1,State::Aborted|State::RecoveryRequired)
                    |(2,State::Aborted|State::RecoveryRequired|State::Committed)) {return Err(Failure::UnsupportedDurability);}
                if candidate.state()==State::Committed && candidate.cause()!=Failure::Interrupted {return Err(Failure::CustodyConflict);}
            }
            if matches!(candidate.state(),State::Aborted|State::RecoveryRequired) && candidate.confirmed() {
                return Err(Failure::CustodyConflict);
            }
            self.outcomes=self.outcomes.checked_add(1).ok_or(Failure::LimitExceeded)?;
            if self.outcomes>self.expected_epoch {return Err(Failure::CustodyConflict);}
            self.terminal=Some(LoadedInitialCustodyV1{record:record.to_vec(),state:candidate.state(),cause:candidate.cause(),ordinal,row_hash:computed});
        }
        self.previous=computed;
        self.next=self.next.checked_add(1).ok_or(Failure::LimitExceeded)?;
        Ok(())
    }
    pub(super) fn finish(self)->Result<LoadedInitialCustodyV1,Failure> {
        use sha2::{Digest,Sha256};
        let last=self.terminal.ok_or(Failure::RecoveryRequired)?;
        if self.outcomes!=self.expected_epoch || <[u8;32]>::from(Sha256::digest(last.record()))!=self.witness_hash {
            return Err(Failure::CustodyConflict);
        }
        Ok(last)
    }
}

/// Actual recovery result, retaining the original cause of the witnessed
/// failed transaction. No adapter result or predicted disposition issues it.
#[cfg(windows)]
pub(super) struct RecoveryOutcomeCustodyRowV1 {ordinal:u8,previous:[u8;32],record:Vec<u8>,hash:[u8;32]}

#[cfg(windows)]
impl RecoveryOutcomeCustodyRowV1 {
    pub(super) fn from_outcome(actual:&super::retained_attempt::UnretainedRecoveryOutcomeV1)->Result<Self,Failure> {
        let prior=actual.start.prior_record();
        let old=RecoveryRecordCandidateV1::decode(prior.record())?;
        let expected=actual.start.expected_payload();let evidence=actual.owner.evidence();
        old.check_binding(&expected.binding,None)?;
        if old.state()!=State::RecoveryRequired || evidence.binding!=expected.binding
            || !old.state().permits(evidence.state()) {return Err(Failure::CustodyConflict);}
        if let Some(failure)=actual.owner.failure() {
            if failure.state!=Some(evidence.state()) || failure.cause!=old.cause()
                || !matches!(evidence.state(),State::Aborted|State::RecoveryRequired) {return Err(Failure::CustodyConflict);}
        }else if evidence.state()!=State::Committed {return Err(Failure::CustodyConflict);}
        let phase=[evidence.state() as u8];let confirmation=[u8::from(evidence.container_confirmed)];
        let mut fields=old.fields;fields[1]=&phase;fields[13]=&confirmation;
        let record=RecoveryRecordCandidateV1::from_fields(fields)?.encode_canonical()?;
        let ordinal=prior.ordinal().checked_add(1).filter(|n|*n<64).ok_or(Failure::LimitExceeded)?;
        let previous=*prior.row_hash();
        let hash=InitialCustodyRowV1::row_hash(&expected.digest,ordinal,&previous,actual.start.expected_raw(),&record)?;
        Ok(Self{ordinal,previous,record,hash})
    }
    pub(super) fn check_outcome(&self,actual:&super::retained_attempt::UnretainedRecoveryOutcomeV1)->Result<(),Failure> {
        let expected=Self::from_outcome(actual)?;
        if self.ordinal!=expected.ordinal || self.previous!=expected.previous || self.record!=expected.record || self.hash!=expected.hash {
            return Err(Failure::CustodyConflict);
        }Ok(())
    }
    pub(super) fn ordinal(&self)->u8 {self.ordinal}
    pub(super) fn previous(&self)->&[u8;32] {&self.previous}
    pub(super) fn record(&self)->&[u8] {&self.record}
    pub(super) fn hash(&self)->&[u8;32] {&self.hash}
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE:&[u8]=include_bytes!("recovery_custody_fixture.bin");

    #[test]
    fn recovery_custody_record_matches_reviewed_vector_without_admission() {
        use sha2::{Digest,Sha256};
        assert_eq!(hex::encode(Sha256::digest(FIXTURE)),"72fc3d20a900b81ce8c22e969a2c7afae25d07d4f89ea080762b5ae9254f3723");
        let candidate=RecoveryRecordCandidateV1::decode(FIXTURE).unwrap();
        assert_eq!(candidate.encode_canonical().unwrap(),FIXTURE);
        let fields=candidate.fields;
        let binding=TransactionBindingV1 {project:fields[2].to_vec(),expected_parent:fields[3].to_vec(),new_revision:fields[4].to_vec(),
            execution_profile:b"VECTOR_ONLY_EXECUTION".to_vec(),pesi:fields[5].try_into().unwrap(),snapshot_sha256:fields[7].try_into().unwrap(),
            snapshot_length:u64::from_be_bytes(fields[6].try_into().unwrap()),revision_integrity:fields[8].try_into().unwrap()};
        candidate.check_binding(&binding,None).unwrap();assert_eq!(encode_prepared_candidate(&binding,None).unwrap(),FIXTURE);
        assert_eq!(candidate.state(),State::Prepared);assert_eq!(candidate.cause(),Failure::Interrupted);assert!(!candidate.confirmed());
        let mut foreign=binding;foreign.execution_profile.push(1);assert!(candidate.check_binding(&foreign,None).is_err());
        // VECTOR_ONLY snapshot/envelope are not admitted runtime/source data.
    }

    #[test]
    fn recovery_custody_record_rejects_malformed_unbounded_and_unknown_tags() {
        for length in [0,1,FIXTURE.len()-1] {assert!(RecoveryRecordCandidateV1::decode(&FIXTURE[..length]).is_err());}
        let mut trailing=FIXTURE.to_vec();trailing.push(0);assert!(RecoveryRecordCandidateV1::decode(&trailing).is_err());
        assert!(matches!(RecoveryRecordCandidateV1::decode(&vec![0;RECORD_MAX+1]),Err(Failure::LimitExceeded)));
        let mut cursor=1;
        for i in 0..14 {
            let length=u32::from_be_bytes(FIXTURE[cursor+1..cursor+5].try_into().unwrap()) as usize;
            let mut duplicate=FIXTURE.to_vec();duplicate[cursor]=0;assert!(RecoveryRecordCandidateV1::decode(&duplicate).is_err());
            let mut oversized=FIXTURE.to_vec();oversized[cursor+1..cursor+5].copy_from_slice(&((MAX_FIELDS[i]+1) as u32).to_be_bytes());
            assert!(matches!(RecoveryRecordCandidateV1::decode(&oversized),Err(Failure::LimitExceeded)));
            if [1,11,12,13].contains(&i) {let mut invalid=FIXTURE.to_vec();invalid[cursor+5]=255;assert!(RecoveryRecordCandidateV1::decode(&invalid).is_err());}
            cursor+=5+length;
        }
    }
}
