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
    pub(super) fn compile(prepared:&ddonirang_lang::proof_authoring::PreparedProofSource, source:&str,path:&str)->Result<Self,ddonirang_lang::ParseError> {
        let mut calls=HashMap::new();
        for candidate in prepared.resolve_identity_execution()? {
            let application=candidate.definition().calculation_application();
            let direct=application.map_or(candidate.definition(),|call|call.definition());
            let initial=ddonirang_proof::CheckedAssumptionTransfer::check_initial_identity(direct)?;
            let checked=initial.admit_direct_calculation(direct)?.with_argument(candidate.argument())?;
            checked.validate_source(source,path)?;
            if let Some(application)=application {
                // Replay the existing conditional caller/callee evidence check.
                // Keep the outer call and its actual inner call as two plans;
                // never replace a consumer body with the terminal operation.
                ddonirang_proof::check_assumption_transfer(candidate.definition())?;
                let consumption=direct.structure_consumption().unwrap();
                let ExprKind::Call{args,..}=&application.call().kind else {unreachable!()};
                let value=args.iter().find(|a|a.resolved_pin.as_deref()==Some(consumption.input().pin_name.as_str()))
                    .ok_or_else(||ddonirang_lang::ParseError{span:application.call().span,
                        message:"E_PROOF_STRUCTURE_CONSUMPTION: 해소된 내부 값 인수가 없습니다".into()})?;
                calls.insert(application.call().id,Self::plan(&checked,application.callee(),&value.expr,source,path)?);
            }
            calls.insert(candidate.call().id,Self::plan(&checked,candidate.seed(),candidate.value(),source,path)?);
        }
        Ok(Self{calls})
    }
    fn plan(checked:&ddonirang_proof::CheckedIdentityConsumption,seed:&SeedDef,value:&Expr,source:&str,path:&str)->Result<IdentityCall,ddonirang_lang::ParseError> {
        let mut seed=seed.clone();
        // Erase only the statically discharged proof parameter at this exact
        // call boundary. The actual body and generic operation stay intact.
        seed.params.retain(|pin| !matches!(&pin.type_ref,TypeRef::Applied{name,..} if name=="보임"));
        if seed.params.len()!=1 || seed.params[0].type_ref!=TypeRef::Named("바른수".into()) {
            return Err(ddonirang_lang::ParseError{span:value.span,
                message:"E_PROOF_STRUCTURE_CONSUMPTION: 실제 바른수 값 핀이 필요합니다".into()});
        }
        Ok(IdentityCall{checked:checked.clone(),source:source.into(),path:path.into(),seed,value:value.clone()})
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
