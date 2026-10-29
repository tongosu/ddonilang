//! R1.1 source-type preimage consumer, not a saved proof or checker verdict.
use super::*;
use ddonirang_lang::{SeedDef, proof_authoring::{prepare_source,PreparedProofSource,VerifiedSavedProofTypeMaterial}};

pub(super) fn contains_goal(t:&TypeRef)->bool {
    match t {
        TypeRef::AlgebraEquality{..}|TypeRef::StructureLaw{..}=>true,
        TypeRef::PropositionConjunction(ts)|TypeRef::Tuple(ts)|TypeRef::Applied{args:ts,..}=>ts.iter().any(contains_goal),
        TypeRef::Tensor{element,..}=>contains_goal(element),
        TypeRef::Function{params,result}=>params.iter().any(contains_goal)||contains_goal(result),
        _=>false,
    }
}
fn rejected()->String {"E_SAVED_DEFINITION_TYPE_UNSUPPORTED: 같은 원문 소유자의 실제 목표 타입 공급이 없습니다".into()}

pub(super) struct SourceTypes {
    prepared:PreparedProofSource,
    facts:Vec<VerifiedSavedProofTypeMaterial>,
}
impl SourceTypes {
    pub(super) fn for_program(canonical:&ddonirang_lang::CanonProgram,source:&str,path:&str)->Result<Option<Self>,String>{
        let needed=canonical.items.iter().any(|TopLevelItem::SeedDef(s)|
            s.params.iter().any(|p|contains_goal(&p.type_ref))||s.result_type.as_ref().is_some_and(contains_goal));
        if !needed{return Ok(None)}
        // Parse the COMPLETE actual prepared source once. Never filter roots,
        // reuse a checker handle, or copy signature facts across a parse owner.
        let prepared=prepare_source(source,path).map_err(|e|format!("E_SAVED_DEFINITION_TYPE_UNSUPPORTED: {e}"))?;
        let facts=prepared.saved_proof_signatures().map_err(|e|format!("E_SAVED_DEFINITION_TYPE_UNSUPPORTED: {e}"))?;
        if facts.iter().any(|fact|!fact.source_matches(source,path)){return Err(rejected())}
        Ok(Some(Self{prepared,facts}))
    }
    pub(super) fn original_definition(&self,canonical:&SeedDef)->Result<&SeedDef,String>{
        self.prepared.program().items.iter().map(|TopLevelItem::SeedDef(s)|s)
            .find(|s|s.id==canonical.id&&s.span==canonical.span&&s.canonical_name==canonical.canonical_name)
            .ok_or_else(rejected)
    }
    pub(super) fn material(&self,canonical:&TypeRef,actual:&TypeRef,seed:&SeedDef)->Result<Value,String>{
        if !contains_goal(canonical){return type_material(canonical)}
        let unsupported=||Err(rejected());
        Ok(match (canonical,actual) {
            (TypeRef::AlgebraEquality{..},TypeRef::AlgebraEquality{..})|
            (TypeRef::StructureLaw{..},TypeRef::StructureLaw{..})=>{
                let mut matching=self.facts.iter().filter(|fact|
                    fact.definition(&self.prepared).is_ok_and(|s|std::ptr::eq(s,seed))
                    &&fact.type_ref(&self.prepared).is_ok_and(|t|std::ptr::eq(t,actual)));
                let fact=matching.next().ok_or_else(rejected)?;
                if matching.next().is_some(){return unsupported()}
                fact.validate_occurrence(&self.prepared,seed,actual).map_err(|_|rejected())?;
                let expected=if matches!(canonical,TypeRef::AlgebraEquality{..}){"algebra_equality"}else{"structure_law"};
                if fact.kind()!=expected||fact.description()!=ddonirang_lang::normalizer::Normalizer::source_type_description(canonical,self.prepared.program().source_epoch){return unsupported()}
                json!(["proof_source_type_v1",fact.kind(),fact.description()])
            },
            (TypeRef::Tuple(cs),TypeRef::Tuple(rs))=>json!(["tuple",self.children(cs,rs,seed)?]),
            (TypeRef::PropositionConjunction(cs),TypeRef::PropositionConjunction(rs))=>json!(["proposition_conjunction",self.children(cs,rs,seed)?]),
            (TypeRef::Applied{name,args},TypeRef::Applied{args:raw,..})=>json!(["applied",name,self.children(args,raw,seed)?]),
            (TypeRef::Tensor{element,shape},TypeRef::Tensor{element:raw,shape:raw_shape})=>{
                if shape!=raw_shape{return unsupported()}json!(["tensor",self.material(element,raw,seed)?,shape])
            },
            (TypeRef::Function{params,result},TypeRef::Function{params:raw_params,result:raw_result})=>
                json!(["function",self.children(params,raw_params,seed)?,self.material(result,raw_result,seed)?]),
            _=>return unsupported(),
        })
    }
    fn children(&self,canonical:&[TypeRef],actual:&[TypeRef],seed:&SeedDef)->Result<Vec<Value>,String>{
        if canonical.len()!=actual.len(){return Err(rejected())}
        canonical.iter().zip(actual).map(|(c,r)|self.material(c,r,seed)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const RAW:&str="(값:나눔수~을, 근거:(값 =:= 값)보임~로) 같음보이다{같음보이기}:(값 =:= 값)보임:셈씨 := { 같음보이다 <- 근거. }.";
    #[test]
    fn wo30_saved_proof_material_uses_actual_occurrence_and_exact_private_array(){
        let mut c=ddonirang_lang::parse_v25_with_mode(RAW,"proof.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
        ddonirang_lang::canonicalize(&mut c).unwrap();
        let ctx=SourceTypes::for_program(&c,RAW,"proof.ddn").unwrap().unwrap();
        let TopLevelItem::SeedDef(canonical)=&c.items[0];let actual=ctx.original_definition(canonical).unwrap();
        let material=ctx.material(&canonical.params[1].type_ref,&actual.params[1].type_ref,actual).unwrap();
        assert_eq!(serde_json::to_string(&material).unwrap(),r#"["applied","보임",[["proof_source_type_v1","algebra_equality","값 =:= 값"]]]"#);
        let cloned=actual.params[1].type_ref.clone();assert!(ctx.material(&canonical.params[1].type_ref,&cloned,actual).is_err());
        let stale=SourceTypes::for_program(&c,RAW,"proof.ddn").unwrap().unwrap();
        assert!(stale.material(&canonical.params[1].type_ref,&actual.params[1].type_ref,actual).is_err());
    }
}
