//! S4 static plans; no runtime proof value or success-JSON constructor.
use super::*;

#[derive(Clone, Default)]
pub(super) struct IdentityCalculations {
    calls: HashMap<NodeId, IdentityCall>,
}
#[derive(Clone)]
pub(super) struct IdentityCall {
    checked: ddonirang_proof::CheckedIdentityConsumption,
    source: String,
    path: String,
    pub(super) seed: SeedDef,
    pub(super) value: Expr,
}
impl IdentityCalculations {
    pub(super) fn compile(prepared:&ddonirang_lang::proof_authoring::PreparedProofSource, source:&str,path:&str)->Result<Self,String> {
        let mut calls=HashMap::new();
        for candidate in prepared.resolve_identity_execution().map_err(|e|format_parse_error(source,&e))? {
            let initial=ddonirang_proof::CheckedAssumptionTransfer::check_initial_identity(candidate.definition())
                .map_err(|e|format_parse_error(source,&e))?;
            let checked=initial.admit_direct_calculation(candidate.definition()).map_err(|e|format_parse_error(source,&e))?;
            checked.validate_source(source,path).map_err(|e|format_parse_error(source,&e))?;
            let mut seed=candidate.seed().clone();
            // Erase only the statically discharged proof parameter. The actual
            // operation call/body and its generic instantiation stay intact.
            seed.params.retain(|pin| !matches!(&pin.type_ref,TypeRef::Applied{name,..} if name=="보임"));
            if seed.params.len()!=1 || seed.params[0].type_ref!=TypeRef::Named("바른수".into()) {
                return Err("E_PROOF_STRUCTURE_CONSUMPTION: 실제 바른수 값 핀이 필요합니다".into());
            }
            calls.insert(candidate.call().id,IdentityCall{checked,source:source.into(),path:path.into(),seed,value:candidate.value().clone()});
        }
        Ok(Self{calls})
    }
    pub(super) fn active(&self)->bool {!self.calls.is_empty()}
    pub(super) fn call(&self,id:NodeId)->Result<Option<IdentityCall>,EvalError> {
        let Some(call)=self.calls.get(&id) else {return Ok(None)};
        call.checked.validate_source(&call.source,&call.path).map_err(|e|EvalError::Message(e.message))?;
        Ok(Some(call.clone()))
    }
    pub(super) fn is_erased_seed(&self,seed:&SeedDef)->bool {
        self.calls.values().any(|call|call.seed.id==seed.id && seed.params.len()==1
            && seed.params[0].id==call.seed.params[0].id && seed.result_type==call.seed.result_type)
    }
}
