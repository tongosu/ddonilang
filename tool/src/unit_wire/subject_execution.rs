//! R5 execution-local staging. This defines no source callable or authority.
use std::{collections::BTreeMap,sync::Arc};
use ddonirang_core::{nominal::{NominalSlotTableV1,SubjectHandleV1},platform::{ResourceValue,SubjectWorldStateV1,IdentityReferenceValueV1}};
use crate::{authored_project::registration::SavedNominalCatalogV1,subject_reference_owner::{LiveSubjectOwnerV1,SubjectReferenceContextV1,TemporarySubjectOwnerV1},snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure};

#[derive(Clone)]
enum ExecutionLifecycle {Saved(LiveSubjectOwnerV1),Temporary(TemporarySubjectOwnerV1)}
impl ExecutionLifecycle {
    fn world(&self)->&[u8;32]{match self{Self::Saved(owner)=>&owner.custody().world,Self::Temporary(owner)=>owner.world()}}
    fn rows(&self)->&[crate::subject_reference_owner::SavedSubjectRow]{match self{Self::Saved(owner)=>&owner.custody().rows,Self::Temporary(owner)=>owner.rows()}}
    fn references(&self)->Result<SubjectReferenceContextV1<'_>,Failure>{match self{Self::Saved(owner)=>SubjectReferenceContextV1::new(owner.custody()),Self::Temporary(owner)=>owner.references()}}
    fn allocate(&mut self,ty:&[u8])->Result<IdentityReferenceValueV1,Failure>{match self{Self::Saved(owner)=>{let pending=owner.prepare_allocation(ty)?;owner.commit_allocation(pending)},Self::Temporary(owner)=>owner.allocate(ty)}}
    fn retire(&mut self,value:&IdentityReferenceValueV1)->Result<(),Failure>{match self{Self::Saved(owner)=>owner.retire(value),Self::Temporary(owner)=>owner.retire(value)}}
}

#[derive(Clone)]
pub(crate) struct SubjectExecutionStage {
    owner:ExecutionLifecycle,
    values:BTreeMap<[u8;32],Arc<NominalSlotTableV1>>,
    catalog:Arc<SavedNominalCatalogV1>,
    atom_evidence:Arc<Vec<u8>>,
}
pub(crate) enum SubjectReadOutcome {
    Snapshot(Arc<ddonirang_lang::runtime::FrozenSubjectViewV1>),
    Broken,
}
impl SubjectExecutionStage {
    pub(crate) fn reference_context(&self)->Result<SubjectReferenceContextV1<'_>,String>{self.owner.references().map_err(error)}
    /// Only called after the actual Native once-session verifies permission.
    /// A cloned transaction stage issues neither a nonce nor an ordinal.
    pub(crate) fn fresh_for_permitted_source(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1)->Result<Option<Self>,String>{
        let Some(registration)=owned.registration() else{return Ok(None)};
        let catalog=registration.saved_nominal_catalog(owned)?;
        if !catalog.schemas().any(|schema|schema.category==1){return Ok(None)}
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
        let owner=LiveSubjectOwnerV1::fresh(*issued.pesi().as_raw()).map_err(error)?;
        let atom_evidence=super::resumable_complete_state::recorded_source_atom_evidence(owned).map_err(error)?;
        Ok(Some(Self{owner:ExecutionLifecycle::Saved(owner),values:BTreeMap::new(),catalog:Arc::new(catalog),atom_evidence:Arc::new(atom_evidence)}))
    }
    pub(crate) fn fresh_for_temporary(context:&crate::authored_project::registration::temporary::TemporaryRegisteredSource,program:&ddonirang_lang::CanonProgram)->Result<Option<Self>,String>{
        let catalog=context.catalog(program)?;
        if !catalog.schemas().any(|s|s.category==1){return Ok(None)}
        let atom_evidence=super::complete_state_tables::encode_registered_atom_evidence(context.contracts()).map_err(error)?;
        Ok(Some(Self{owner:ExecutionLifecycle::Temporary(TemporarySubjectOwnerV1::fresh(context.digest()).map_err(error)?),
            values:BTreeMap::new(),catalog:Arc::new(catalog),atom_evidence:Arc::new(atom_evidence)}))
    }
    pub(crate) fn stage_committed(&self,snapshot:&SubjectWorldStateV1)->Result<Self,String>{
        if let ExecutionLifecycle::Temporary(previous)=&self.owner {
            let owner=previous.restage(&snapshot.subject_custody).map_err(error)?;
            let contracts=super::complete_state_tables::decode_registered_atom_evidence(&self.atom_evidence).map_err(error)?;
            let refs=owner.references().map_err(error)?;
            let rows=super::complete_state_tables::decode_temporary_subject_values(&snapshot.subject_values,owner.rows(),&contracts,&refs,&self.catalog).map_err(error)?;
            let mut values=BTreeMap::new();
            for (id,row) in rows {let ResourceValue::SubjectPayloadV1(value)=&row.value else{return Err(error(Failure::SubjectCustodyInvalid))};values.insert(id,Arc::clone(value));}
            return Ok(Self{owner:ExecutionLifecycle::Temporary(owner),values,catalog:Arc::clone(&self.catalog),atom_evidence:Arc::clone(&self.atom_evidence)})
        }
        let ExecutionLifecycle::Saved(previous)=&self.owner else{unreachable!()};
        let empty=[0;8];
        let frame=super::complete_state_frame::CompleteStateFrameV1{pesi:&previous.custody().pesi,
            resources:&empty,parameters:&empty,next_tick:0,subjects:&snapshot.subject_values,
            atom_evidence:&self.atom_evidence,subject_custody:&snapshot.subject_custody};
        let tables=frame.decode_value_tables_with_catalog(Some(&self.catalog)).map_err(error)?;
        let owner=LiveSubjectOwnerV1::from_recorded_candidate(tables.subject_custody.ok_or_else(||error(Failure::SubjectCustodyInvalid))?).map_err(error)?;
        if &owner.custody().world!=self.owner.world(){return Err(error(Failure::SubjectCustodyInvalid))}
        let mut values=BTreeMap::new();
        for (id,row) in tables.subjects {
            let ResourceValue::SubjectPayloadV1(value)=&row.value else{return Err(error(Failure::SubjectCustodyInvalid))};
            values.insert(id,Arc::clone(value));
        }
        Ok(Self{owner:ExecutionLifecycle::Saved(owner),values,catalog:Arc::clone(&self.catalog),atom_evidence:Arc::clone(&self.atom_evidence)})
    }
    /// Recorded initial data for private replay staging, not a Native permit.
    /// No fresh entropy or repair is allowed while rebuilding this candidate.
    pub(crate) fn from_recorded_initial(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        frame:&super::complete_state_frame::CompleteStateFrameV1<'_>)->Result<Option<Self>,String>{
        let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose()?;
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)?;
        if frame.pesi!=issued.pesi().as_raw(){return Err(error(Failure::CustodyConflict))}
        let atom_evidence=super::resumable_complete_state::recorded_source_atom_evidence(owned).map_err(error)?;
        if frame.atom_evidence!=atom_evidence{return Err(error(Failure::AtomEvidenceInvalid));}
        let values=frame.decode_value_tables_with_catalog(catalog.as_ref()).map_err(error)?;
        let Some(custody)=values.subject_custody else{return Ok(None)};
        if frame.next_tick!=0||!values.resources.is_empty()||!values.parameters.is_empty()||!values.subjects.is_empty(){return Err(error(Failure::InvalidInput))}
        if custody.next_subject_ordinal!=0||!custody.rows.is_empty(){return Err(error(Failure::SubjectCustodyInvalid))}
        let catalog=catalog.filter(|catalog|catalog.schemas().any(|schema|schema.category==1))
            .ok_or_else(||error(Failure::SubjectCustodyInvalid))?;
        Ok(Some(Self{owner:ExecutionLifecycle::Saved(LiveSubjectOwnerV1::from_recorded_candidate(custody).map_err(error)?),
            values:BTreeMap::new(),catalog:Arc::new(catalog),atom_evidence:Arc::new(atom_evidence)}))
    }
    pub(crate) fn create(&mut self,payload:Arc<NominalSlotTableV1>)->Result<SubjectHandleV1,String>{
        if self.catalog.schema(payload.name_type_identity()).is_none_or(|s|s.category!=1){return Err(error(Failure::SubjectCustodyInvalid))}
        let mut candidate=self.clone();
        let reference=candidate.owner.allocate(payload.name_type_identity()).map_err(error)?;
        let handle=SubjectHandleV1{name_type_identity:*payload.name_type_identity(),world_identity:*candidate.owner.world(),
            subject_identity:reference.target_identity.as_slice().try_into().map_err(|_|error(Failure::SubjectCustodyInvalid))?};
        candidate.values.insert(handle.subject_identity,payload);
        candidate.committed_snapshot()?;*self=candidate;Ok(handle)
    }
    pub(crate) fn retire(&mut self,reference:&IdentityReferenceValueV1)->Result<(),String>{
        let mut candidate=self.clone();candidate.owner.retire(reference).map_err(error)?;
        let id:&[u8;32]=reference.target_identity.as_slice().try_into().map_err(|_|error(Failure::SubjectCustodyInvalid))?;
        if candidate.values.remove(id).is_none(){return Err(error(Failure::SubjectCustodyInvalid))}
        candidate.committed_snapshot()?;*self=candidate;Ok(())
    }
    pub(crate) fn acquire(&self,handle:&SubjectHandleV1)->Result<IdentityReferenceValueV1,String>{
        self.owner.references().and_then(|context|context.verify_handle(handle)).map_err(error)?;
        Ok(IdentityReferenceValueV1{profile_version:1,target_type_identity:handle.name_type_identity.to_vec(),
            owner_space_identity:self.owner.world().to_vec(),target_identity:handle.subject_identity.to_vec(),generation:0})
    }
    pub(crate) fn read(&self,reference:&IdentityReferenceValueV1,expected:&[u8;32])->Result<SubjectReadOutcome,String>{
        self.committed_snapshot()?;
        // Only the actually issued generation-zero profile is admitted. A
        // malformed/missing-row value is not a fabricated semantic tombstone.
        if reference.profile_version!=1||reference.generation!=0||reference.owner_space_identity.as_slice()!=self.owner.world()
            ||reference.target_type_identity.as_slice()!=expected {return Err(error(Failure::SubjectCustodyInvalid));}
        let id:&[u8;32]=reference.target_identity.as_slice().try_into().map_err(|_|error(Failure::SubjectCustodyInvalid))?;
        let row=self.owner.rows().iter().find(|row|&row.identity==id).ok_or_else(||error(Failure::SubjectCustodyInvalid))?;
        if row.target_type.as_slice()!=expected{return Err(error(Failure::SubjectCustodyInvalid));}
        if !row.live{return Ok(SubjectReadOutcome::Broken);}
        let table=self.values.get(id).cloned().ok_or_else(||error(Failure::SubjectCustodyInvalid))?;
        let references=self.owner.references().map_err(error)?;
        let contracts=super::complete_state_tables::decode_registered_atom_evidence(&self.atom_evidence).map_err(error)?;
        let view=super::saved_resource_value::capture_subject_view(table,&self.catalog,&references,&contracts).map_err(error)?;
        Ok(SubjectReadOutcome::Snapshot(Arc::new(view)))
    }
    pub(crate) fn replace_payload(&mut self,handle:&SubjectHandleV1,payload:Arc<NominalSlotTableV1>)->Result<(),String>{
        self.owner.references().and_then(|context|context.verify_handle(handle)).map_err(error)?;
        if payload.name_type_identity()!=&handle.name_type_identity{return Err(error(Failure::SubjectCustodyInvalid))}
        let mut candidate=self.clone();
        if candidate.values.insert(handle.subject_identity,payload).is_none(){return Err(error(Failure::SubjectCustodyInvalid))}
        candidate.committed_snapshot()?;*self=candidate;Ok(())
    }
    pub(crate) fn committed_snapshot(&self)->Result<SubjectWorldStateV1,String>{
        let mut length=8usize;
        for value in self.values.values(){length=length.checked_add(41).and_then(|n|n.checked_add(value.canonical_body().len()))
            .filter(|n|*n<=2_000_000).ok_or_else(||error(Failure::LimitExceeded))?;}
        let mut subjects=Vec::new();subjects.try_reserve_exact(length).map_err(|_|error(Failure::LimitExceeded))?;
        subjects.extend((self.values.len() as u64).to_be_bytes());
        for (id,value) in &self.values {
            subjects.extend(id);subjects.extend(((value.canonical_body().len()+1) as u64).to_be_bytes());
            subjects.push(0x13);subjects.extend(value.canonical_body());
        }
        let custody=match &self.owner {ExecutionLifecycle::Saved(owner)=>super::complete_state_tables::encode_saved_world_custody(owner.custody()),ExecutionLifecycle::Temporary(owner)=>owner.encode()}.map_err(error)?;
        let snapshot=SubjectWorldStateV1{subject_values:subjects,subject_custody:custody};
        if matches!(self.owner,ExecutionLifecycle::Temporary(_)){self.stage_committed(&snapshot)?;return Ok(snapshot)}
        let ExecutionLifecycle::Saved(owner)=&self.owner else{unreachable!()};
        let empty=[0;8];
        super::complete_state_frame::CompleteStateFrameV1{pesi:&owner.custody().pesi,
            resources:&empty,parameters:&empty,next_tick:0,subjects:&snapshot.subject_values,
            atom_evidence:&self.atom_evidence,subject_custody:&snapshot.subject_custody}
            .encode_canonical_with_catalog(Some(&self.catalog)).map_err(error)?;
        Ok(snapshot)
    }
}
fn error(cause:Failure)->String{format!("E_VC04_SUBJECT_STATE: 대상 상태·수명 결속이 올바르지 않습니다 ({cause:?})")}
