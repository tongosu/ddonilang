use ddonirang_lang::{proof_authoring::{prepare_source,SavedProofSignatureSlot,SavedProofTypeChild},ast::*,normalizer::Normalizer};
fn algebra()->String {
    "(값:나눔수~을, 근거:(값 =:= 값)보임~로)
같음보이다{같음보이기}:(값 =:= 값)보임:셈씨 := { 같음보이다 <- 근거. }.".into()
}
#[test]
fn header_fact_binds_actual_definition_leaf_and_normalizer_description() {
    let raw=algebra();let prepared=prepare_source(&raw,"보임.ddn").unwrap();
    let facts=prepared.saved_proof_signatures().unwrap();assert_eq!(facts.len(),2);
    for fact in &facts {
        let seed=fact.definition(&prepared).unwrap();let ty=fact.type_ref(&prepared).unwrap();
        fact.validate_occurrence(&prepared,seed,ty).unwrap();
        assert_eq!(fact.kind(),"algebra_equality");assert_eq!(fact.description(),"값 =:= 값");
        assert_eq!(fact.description(),Normalizer::source_type_description(ty,prepared.program().source_epoch));
        assert_eq!(fact.algebra_parameters().unwrap()[0].id,seed.params[0].id);
        assert!(fact.algebra_goal().is_some());assert!(fact.structure_law().is_none());
        assert_eq!(fact.children(),&[SavedProofTypeChild::Argument(0)]);
        assert!(fact.validate_occurrence(&prepared,seed,&ty.clone()).is_err());
        assert!(fact.validate_occurrence(&prepared,&seed.clone(),ty).is_err());
    }
    assert_eq!(facts[0].slot(),&SavedProofSignatureSlot::Input(1));assert_eq!(facts[1].slot(),&SavedProofSignatureSlot::Result);
}
#[test]
fn full_root_and_invalid_proof_body_are_not_filtered_or_marked_checked() {
    let raw=format!("1 보여주기.\n{}",algebra().replace("같음보이다 <- 근거","같음보이다 <- 참"));
    let prepared=prepare_source(&raw,"전체.ddn").unwrap();let root_count=prepared.program().root_stmts.len();assert_eq!(root_count,1);
    assert_eq!(prepared.saved_proof_signatures().unwrap().len(),2);
    assert_eq!(prepared.program().root_stmts.len(),root_count);
    assert!(prepared.resolve_algebra().unwrap_err().message.starts_with("E_PROOF_ROOT_UNSUPPORTED"));
    let body_only=raw.replace("1 보여주기.\n","");let p=prepare_source(&body_only,"본문.ddn").unwrap();
    assert_eq!(p.saved_proof_signatures().unwrap().len(),2);assert!(p.resolve_algebra().is_err());
}
#[test]
fn unknown_and_nonrational_binders_rejected_without_issuing_fact() {
    for raw in [algebra().replace("값 =:= 값","없는값 =:= 없는값"),algebra().replace("값:나눔수","값:글")] {
        let p=prepare_source(&raw,"미결.ddn").unwrap();assert!(p.saved_proof_signatures().is_err());
    }
    let raw=format!("나눔수 := 1.\n{}",algebra());
    let p=prepare_source(&raw,"가림.ddn").unwrap();assert!(p.saved_proof_signatures().is_err());
}
#[test]
fn identical_reparse_stale_source_and_other_definition_cannot_reuse_fact() {
    let raw=format!("{}\n{}",algebra(),algebra().replace("같음보이다","다른보이다").replace("같음보이기","다른보이기"));
    let p=prepare_source(&raw,"same.ddn").unwrap();let q=prepare_source(&raw,"same.ddn").unwrap();
    let facts=p.saved_proof_signatures().unwrap();
    assert!(facts[0].definition(&q).is_err());
    assert!(!facts[0].source_matches(&(raw.clone()+"\n"),"same.ddn"));
    assert!(!facts[0].source_matches(&raw,"different.ddn"));
    assert!(facts[0].validate_occurrence(&p,facts[2].definition(&p).unwrap(),facts[2].type_ref(&p).unwrap()).is_err());
    assert!(facts[0].validate_occurrence(&p,facts[1].definition(&p).unwrap(),facts[1].type_ref(&p).unwrap()).is_err());
}
#[test]
fn structure_fact_preserves_real_operation_dependency_and_root_context() {
    let raw=format!("{}\n1 보여주기.",include_str!("structure_law_ko_surface_r1.ddn"));
    let p=prepare_source(&raw,"구조.ddn").unwrap();let facts=p.saved_proof_signatures().unwrap();assert_eq!(facts.len(),3);
    assert!(facts[0].structure_law().unwrap().same_structure(facts[1].structure_law().unwrap()));
    for f in facts {
        assert_eq!(f.kind(),"structure_law");let law=f.structure_law().unwrap();
        assert_eq!(law.operation().canonical_name,"바른수그대로");assert_eq!(law.dependency().canonical_name,"그대로");
        assert_eq!(law.domain(),&TypeRef::Named("바른수".into()));
        f.validate_occurrence(&p,f.definition(&p).unwrap(),f.type_ref(&p).unwrap()).unwrap();
    }
    assert!(p.resolve().is_err());assert_eq!(p.program().root_stmts.len(),2);
}
#[test]
fn unresolved_external_structure_and_wrong_closure_rejected() {
    let missing="(근거:(외부구조.법칙)보임~을) 전달:(외부구조.법칙)보임:셈씨 := { 전달 <- 근거. }.";
    assert!(prepare_source(missing,"외부.ddn").unwrap().saved_proof_signatures().is_err());
    let raw=include_str!("structure_law_ko_surface_r1.ddn").replace("그대로 <- 값.","그대로 <- 1.");
    // Parser may already reject incompatible generic body; otherwise signature must.
    match prepare_source(&raw,"잘못된의존.ddn") {Ok(p)=>assert!(p.saved_proof_signatures().is_err()),Err(_)=>{}}
}
#[test]
fn nested_occurrences_have_distinct_routes_and_do_not_change_old_types() {
    let raw="(값:나눔수~을, 근거:(값 =:= 값)보임~로) 모으다:((값 =:= 값)보임, (값 =:= 값)보임):셈씨 := { 모으다 <- (근거, 근거). }.";
    let p=prepare_source(raw,"중첩.ddn").unwrap();let facts=p.saved_proof_signatures().unwrap();assert_eq!(facts.len(),3);
    assert_eq!(facts[1].children(),&[SavedProofTypeChild::Tuple(0),SavedProofTypeChild::Argument(0)]);
    assert_eq!(facts[2].children(),&[SavedProofTypeChild::Tuple(1),SavedProofTypeChild::Argument(0)]);
    assert!(facts[1].validate_occurrence(&p,facts[2].definition(&p).unwrap(),facts[2].type_ref(&p).unwrap()).is_err());
    let simple=prepare_source("(값:바른수~을) 그대로:바른수:셈씨 := { 그대로 <- 값. }.","보통.ddn").unwrap();
    assert!(simple.saved_proof_signatures().unwrap().is_empty());
    assert_eq!(Normalizer::source_type_description(&TypeRef::Named("바른수".into()),simple.program().source_epoch),"바른수");
}

#[test]
fn structure_source_shadow_and_equal_description_from_other_owner_do_not_merge() {
    let raw=include_str!("structure_law_ko_surface_r1.ddn");
    let p=prepare_source(raw,"하나.ddn").unwrap();let q=prepare_source(raw,"둘.ddn").unwrap();
    let pf=p.saved_proof_signatures().unwrap();let qf=q.saved_proof_signatures().unwrap();
    assert_eq!(pf[0].description(),qf[0].description());
    assert!(!pf[0].structure_law().unwrap().same_structure(qf[0].structure_law().unwrap()));
    assert!(pf[0].definition(&q).is_err());
    let shadow=format!("바른수 := 1.\n{raw}");
    match prepare_source(&shadow,"가림.ddn") {Ok(p)=>assert!(p.saved_proof_signatures().is_err()),Err(_)=>{}}
}

#[test]
fn v4_division_body_does_not_issue_a_checked_saved_header() {
    let raw=include_str!("../../tool/tests/fixtures/v4_linear_steps/정상.ddn");
    for source in [raw.to_owned(),raw.replace("오른=6","오른=5"),raw.replace("계수=3 / 2","계수=2 - 2")] {
        let p=prepare_source(&source,"수업.ddn").unwrap();
        let facts=p.saved_proof_signatures().unwrap();assert_eq!(facts.len(),2);
        assert!(facts.iter().all(|f|f.kind()=="algebra_equality"));
        let other=prepare_source(&source,"수업.ddn").unwrap();
        assert!(facts[0].definition(&other).is_err());
        assert!(!facts[0].source_matches(&(source.clone()+"\n"),"수업.ddn"));
    }
}
