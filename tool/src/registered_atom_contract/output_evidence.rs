//! Output-only projection from the actual admitted source/registration owner.
//! This is not a TypeRef, saved slot admission or runtime-shape type issuer.
use std::{collections::{BTreeMap,HashSet},sync::Arc};
use ddonirang_core::platform::RecordFieldType;
use ddonirang_lang::{CanonProgram,NodeId,runtime::Value};
use crate::authored_project::registration::ResolvedRegisteredAtomResultTypeV1;

#[derive(Debug)]
pub(crate) struct OutputTypePlanV1 {
    positions:BTreeMap<NodeId,RecordFieldType>,
    source:Arc<ResolvedRegisteredAtomResultTypeV1>,
}
impl OutputTypePlanV1 {
    pub(crate) fn compile(program:&CanonProgram,world:&HashSet<String>,host:Option<&crate::runtime_surface::BogaeHostNumericSupply>,
        original:&crate::numeric_constraint_plan::NumericConstraintPlan,source:ResolvedRegisteredAtomResultTypeV1)->Option<Arc<Self>>{
        let projection=crate::numeric_constraint_plan::NumericConstraintPlan::compile_output_with_atoms(program,world,host,&source).ok()?;
        // The extra source projection may resolve formerly unknown atom
        // leaves, never change an already-owned numeric/nominal output type.
        for (site,kind)in &original.position_types {
            if projection.position_types.get(site)!=Some(kind){return None;}
        }
        Some(Arc::new(Self{positions:projection.position_types,source:Arc::new(source)}))
    }
    pub(crate) fn kind(&self,site:NodeId)->Option<&RecordFieldType>{self.positions.get(&site)}
    pub(crate) fn source(&self)->&ResolvedRegisteredAtomResultTypeV1{&self.source}
}

pub(crate) struct OutputValueContextV1<'a> {
    pub(crate) source:Option<&'a ResolvedRegisteredAtomResultTypeV1>,
    pub(crate) contracts:&'a [super::RecordedAtomContract],
}
impl OutputValueContextV1<'_>{
    pub(crate) fn registered_atom(&self,value:&Value)->Result<ddonirang_core::platform::ResourceValue,String>{
        let source=self.source.ok_or("E_UNSUPPORTED_RESULT_EVIDENCE: 원자 출력의 실제 원문 등록 증거가 없습니다")?;
        let Value::RegisteredAtom(value)=value else{return Err("E_UNSUPPORTED_RESULT_EVIDENCE: 원자 출력씨와 값 carrier가 다릅니다".into());};
        source.verify_value(value,self.contracts)?;
        Ok(ddonirang_core::platform::ResourceValue::RegisteredAtom(value.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ddn_runtime::DdnProgram,authored_project::registration::temporary::TemporaryRegisteredSource};

    #[test]
    fn b3_atom_output_actual_source_proof_rejects_missing_foreign_and_stale_contracts() {
        let program=DdnProgram::from_v25_source("표지 := #표지. 표지 보여주기.","atom-proof.ddn").unwrap();
        let owner=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        let (atoms,proof)=owner.bind_atoms(program.canonical_program()).unwrap().into_parts();
        let atom=atoms.values().next().unwrap();
        let value=Value::RegisteredAtom(Box::new(atom.clone()));
        let context=OutputValueContextV1{source:Some(&proof),contracts:owner.contracts()};
        assert!(context.registered_atom(&value).is_ok());
        assert!(context.registered_atom(&Value::Bool(true)).is_err());
        assert!(OutputValueContextV1{source:None,contracts:owner.contracts()}.registered_atom(&value).is_err());
        assert!(OutputValueContextV1{source:Some(&proof),contracts:&[]}.registered_atom(&value).is_err());
        let foreign=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        assert!(OutputValueContextV1{source:Some(&proof),contracts:foreign.contracts()}.registered_atom(&value).is_err());
        let mut stale=atom.clone();stale.contract_sha256[0]^=1;
        assert!(context.registered_atom(&Value::RegisteredAtom(Box::new(stale))).is_err());
        let changed=DdnProgram::from_v25_source("표지 := #다른표지. 표지 보여주기.","atom-proof.ddn").unwrap();
        assert!(!proof.matches_program(changed.canonical_program()));

        // A valid prefix and a later invalid emission cannot publish part
        // of the next madi or consume its ordinal.
        let mut sink=crate::replay_evidence::ResultEmissionSinkV1::default();
        let empty=sink.prepare_madi(Default::default(),0).unwrap();
        sink.require_exact(&empty).unwrap();sink.commit_madi(empty).unwrap();
        let before=sink.content_bytes().unwrap();
        let mut batch=crate::replay_evidence::StagedResultEmissionsV1::default();
        batch.stage(1,Some(&RecordFieldType::RegisteredAtomV1),&BTreeMap::new(),&value,Some(&proof),owner.contracts());
        batch.stage(1,Some(&RecordFieldType::RegisteredAtomV1),&BTreeMap::new(),&Value::Bool(true),Some(&proof),owner.contracts());
        let pending=sink.prepare_madi(batch,1).unwrap();
        assert!(sink.require_exact(&pending).is_err());
        assert_eq!(sink.next_madi(),1);assert_eq!(sink.content_bytes().unwrap(),before);
    }

    #[test]
    fn b3_atom_output_policy_spelling_never_supplies_local_atom_type() {
        let program=DdnProgram::from_v25_source("색 <- #오른쪽. 곁 := (보기=(() 표준보기.결과표),자리=#오른쪽) 표준보개.곁.","atom-role.ddn").unwrap();
        let owner=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        let (atoms,proof)=owner.bind_atoms(program.canonical_program()).unwrap().into_parts();
        assert_eq!(atoms.len(),1);
        let mut local=0;let mut policy=0;
        for (node,site)in &program.canonical_program().origin.atom_value_occurrences {
            match site.role {
                ddonirang_lang::ast::AtomValueRole::LocalValue=>{assert!(proof.supplies(program.canonical_program(),*node));local+=1;},
                _=>{assert!(!proof.supplies(program.canonical_program(),*node));policy+=1;},
            }
        }
        assert_eq!((local,policy),(1,1));
    }
}
